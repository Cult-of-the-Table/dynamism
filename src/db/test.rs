use crate::db::worker::work;
use crate::segmentation::model::EmbeddedChunk;
use anyhow::Result;
use arrow_schema::{DataType, Field, Schema};
use fastembed::Embedding;
use std::sync::Arc;
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

    let db = lancedb::connect(("../".to_owned() + dir.path().to_str().unwrap()).as_str())
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
        .create_empty_table("test".to_string(), schema.clone())
        .execute()
        .await
        .unwrap();

    work(schema.clone(), &table, v).await;

    let row_count = table.count_rows(None).await.unwrap();
    assert_eq!(row_count, 1, "Table should contain only 1 row");
    Ok(())
}
