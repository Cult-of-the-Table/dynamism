use crate::umap::FittedChunks;
use arrow_array::types::Float64Type;
use arrow_array::{FixedSizeListArray, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::Table;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub async fn work(schema: Arc<Schema>, table: &Table, chunks: Vec<FittedChunks>) {
    let embeds = FixedSizeListArray::from_iter_primitive::<Float64Type, _, _>(
        chunks.iter().map(|s| {
            let v: [f64; 2] = s.embeds.into();
            Some(v.into_iter().map(Some))
        }),
        2,
    );
    let urls = StringArray::from_iter_values(chunks.iter().map(|s| s.url.to_string()));
    let snippet = StringArray::from_iter_values(chunks.iter().map(|s| s.snippet.to_string()));
    let text = StringArray::from_iter_values(chunks.iter().map(|s| s.text.to_string()));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(embeds),
            Arc::new(urls),
            Arc::new(snippet),
            Arc::new(text),
        ],
    )
    .unwrap();

    table.add(vec![batch]).execute().await.unwrap();
}

pub fn spawn(chunks: Vec<FittedChunks>, dir: String, name: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let db = lancedb::connect((&dir).as_str()).execute().await.unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new(
                "embeds",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, true)), 2),
                true,
            ),
            Field::new("url", DataType::Utf8, false),
            Field::new("snippet", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
        ]));
        let table = db
            .create_empty_table(name, schema.clone())
            .execute()
            .await
            .unwrap();

        work(schema.clone(), &table, chunks).await;
    })
}
