use crate::segmentation::model::EmbeddedChunk;
use crate::segmentation::worker::model::EmbeddingResponse;
use crate::telemetry::TelEvent;
use anyhow::Result;
use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::Table;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub async fn work(schema: Arc<Schema>, table: &Table, chunks: Vec<EmbeddedChunk>) {
    let embeds = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        chunks.iter().map(|s| {
            let v = s.embedding.to_vec();
            Some(v.into_iter().map(Some))
        }),
        768,
    );
    let urls = StringArray::from_iter_values(chunks.iter().map(|s| s.source_url.to_string()));
    let text = StringArray::from_iter_values(chunks.iter().map(|s| s.source_text.to_string()));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(embeds), Arc::new(urls), Arc::new(text)],
    )
    .unwrap();

    let batch_iter = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema.clone());

    table.add(batch_iter).execute().await.unwrap();
}

pub fn spawn(
    mut input_channel: Receiver<Result<EmbeddingResponse>>,
    dir: String,
    name: String,
    tel: Sender<TelEvent>,
) -> JoinHandle<()> {
    let handle = tokio::spawn(async move {
        let db = lancedb::connect(("../".to_owned() + &dir).as_str())
            .execute()
            .await
            .unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new(
                "embedding",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 768),
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

        while let Some(Ok(EmbeddingResponse { chunks })) = input_channel.recv().await {
            let count = chunks.len();
            work(schema.clone(), &table, chunks).await;
            tel.send(TelEvent::ProcessedData(count as u64))
                .await
                .unwrap();
        }
    });
    handle
}
