use anyhow::Result;
use dynamism::db::worker::spawn;
use dynamism::reqwest::download;
use dynamism::scraper::parse;
use dynamism::segmentation::worker::model::EmbeddingTask;
use dynamism::websearch::search;
use tempfile::tempdir;
use tokio::task::JoinSet;
#[tokio::test]
async fn init_pipe() -> Result<()> {
    let query = "rust language";

    let mut set = JoinSet::new();
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
    let task = EmbeddingTask {
        source_text: parse.0,
        url: parse.1,
    };

    let (tx, rx, seg_handle) = dynamism::segmentation::worker::spawn();
    tx.send(task).await.unwrap();
    drop(tx);
    let dir = tempdir()?;
    let db_handle = spawn(
        rx,
        dir.path().to_str().unwrap().to_string(),
        "test".to_string(),
    );
    db_handle.await.unwrap();
    seg_handle.await.unwrap();
    Ok(())
}
