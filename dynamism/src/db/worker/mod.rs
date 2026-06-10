use crate::umap::FittedChunks;
use arrow_array::types::Float64Type;
use arrow_array::{FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
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
    let text = StringArray::from_iter_values(chunks.iter().map(|s| s.text.to_string()));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(embeds), Arc::new(urls), Arc::new(text)],
    )
    .unwrap();

    let batch_iter = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema.clone());

    table.add(batch_iter).execute().await.unwrap();
}

pub fn spawn(chunks: Vec<FittedChunks>, dir: String, name: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let db = lancedb::connect(("../".to_owned() + &dir).as_str())
            .execute()
            .await
            .unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new(
                "embedding",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, true)), 2),
                true,
            ),
            Field::new("url", DataType::Utf8, false),
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
