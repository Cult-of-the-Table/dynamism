use crate::segmentation::model::EmbeddedChunk;
use arrow_array::{
    FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray, types::Float32Type,
};
use arrow_schema::{DataType, Field, Schema};
use std::sync::Arc;

pub async fn data(chunks: Vec<EmbeddedChunk>, dir: &str, name: String) {
    let db = lancedb::connect(("../".to_owned() + dir).as_str())
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
    let embeds = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        chunks.iter().map(|s| {
            let v = s.embedding.to_vec();
            Some(v.into_iter().map(Some))
        }),
        768,
    );
    let urls = StringArray::from_iter_values(chunks.iter().map(|s| s.source_url.to_string()));
    let text = StringArray::from_iter_values(chunks.iter().map(|s| s.source_text.to_string()));
    println!("ingest successful");
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(embeds), Arc::new(urls), Arc::new(text)],
    )
    .unwrap();
    let batch_iter = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema.clone());
    db.create_table(name, Box::new(batch_iter))
        .execute()
        .await
        .unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use fastembed::Embedding;
    use tempfile::tempdir;
    use tokio;
    #[tokio::test]
    async fn init() -> Result<()> {
        let mut v: Embedding = vec![-0.04215693, -0.0008360635, -0.06397502, 0.005060206];
        v.resize(768, 0.0);
        let embed = EmbeddedChunk {
            embedding: v,
            ..Default::default()
        };
        let v: Vec<EmbeddedChunk> = vec![embed];
        let dir = tempdir()?;
        data(v, dir.path().to_str().unwrap(), ("test").to_string()).await;
        let db = lancedb::connect(("../".to_owned() + dir.path().to_str().unwrap()).as_str())
            .execute()
            .await?;
        let table = db.open_table("test").execute().await.unwrap();
        let row_count = table.count_rows(None).await.unwrap();
        assert_eq!(row_count, 1, "Table should contain only 1 row");
        Ok(())
    }
}
