use anyhow::Result;
use main::db::data;
use main::reqwest::download;
use main::scraper::parse;
use main::segmentation::{chunker, model::EmbeddedChunk};
use main::websearch::search;
use tempfile::tempdir;
use tokio::task::JoinSet;
#[tokio::test]
async fn init() -> Result<()> {
    let query = "rust language";

    let results = search(query).await?;
    let response = download(results).await?;
    let parse = parse(response).await?;
    let mut set = JoinSet::new();
    parse.into_iter().for_each(|s| {
        set.spawn(async move {
            let (s, u) = s;
            chunker(s.as_str(), u.as_str(), 0.1).await.unwrap()
        });
    });
    let mut chunks: Vec<Vec<EmbeddedChunk>> = Vec::new();
    while let Some(res) = set.join_next().await {
        let s = res?;
        chunks.push(s);
    }
    let chunks = chunks.into_iter().flatten().collect::<Vec<EmbeddedChunk>>();
    let dir = tempdir()?;
    data(chunks, dir.path().to_str().unwrap(), ("test").to_string()).await;
    let db = lancedb::connect(("../".to_owned() + dir.path().to_str().unwrap()).as_str())
        .execute()
        .await?;
    let table = db.open_table("test").execute().await.unwrap();
    let row_count = table.count_rows(None).await.unwrap();
    Ok(())
}
