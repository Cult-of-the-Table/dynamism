use arrow_array::{FixedSizeListArray, RecordBatch, RecordBatchIterator, types::Float32Type};
use arrow_schema::{DataType, Field, Schema};
use fastembed::Embedding;
use std::sync::Arc;

pub async fn data(embeds: Vec<Embedding>, dir: &str) {
    let db = lancedb::connect(("../".to_owned() + dir).as_str())
        .execute()
        .await
        .unwrap();
    let schema = Arc::new(Schema::new(vec![Field::new(
        "embedding",
        DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 768),
        true,
    )]));
    let data = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        embeds.iter().map(|s| {
            let v = s.to_vec();
            Some(v.into_iter().map(Some))
        }),
        768,
    );
    println!("ingest successful");
    let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(data)]).unwrap();
    let batch_iter = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema.clone());
    db.create_table("test", Box::new(batch_iter))
        .execute()
        .await
        .unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tempfile::tempdir;
    use tokio;
    #[tokio::test]
    async fn init() -> Result<()> {
        let mut v: Embedding = vec![-0.04215693, -0.0008360635, -0.06397502, 0.005060206];
        v.resize(768, 0.0);
        let v: Vec<Embedding> = vec![v];
        let dir = tempdir();
        data(v, dir?.path().to_str().unwrap()).await;
        Ok(())
    }
}
