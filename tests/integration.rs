use anyhow::Result;
use futures::{TryStreamExt, stream};
use lancedb::index::Index;
use lancedb::query::{self, ExecutableQuery};
use main::db::data;
use main::reqwest::download;
use main::scraper::parse;
use main::segmentation::{chunker, model::EmbeddedChunk};
use main::websearch::search;
use tempfile::tempdir;
use tokio::task::JoinSet;
#[tokio::test]
async fn init_pipe() -> Result<()> {
    let query = "rust language";

    let results = search(query).await?;
    let response = download(results).await?;

    for (s, u) in response {
        let s = s.text().await.unwrap();
        let u = u.to_string();
        set.spawn(async move { parse((s, u)).await });
    }
    let mut parse = (String::new(), String::new());
    while let Some(res) = set.join_next().await {
        parse = res??;
    }
    let mut set = JoinSet::new();
    let (s, u) = parse;
    let chunks = chunker(s.as_str(), u.as_str(), 0.1).await.unwrap();
    let dir = tempdir()?;
    let first_chunk = chunks.first().map(|s| s.embedding.clone());
    data(chunks, dir.path().to_str().unwrap(), ("test").to_string()).await;
    let db = lancedb::connect(("../".to_owned() + dir.path().to_str().unwrap()).as_str())
        .execute()
        .await?;
    let table = db.open_table("test").execute().await.unwrap();
    table
        .create_index(&["embedding"], Index::Auto)
        .execute()
        .await?;
    let row_count = table.count_rows(None).await.unwrap();
    let results = table
        .query()
        .nearest_to(first_chunk.unwrap())?
        .execute()
        .await?
        .try_collect::<Vec<_>>()
        .await?;
    println!("Search results: {:?}", results);
    Ok(())
}
