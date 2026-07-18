use anyhow::Result;
use dynamism::db::worker::spawn;
use dynamism::reqwest::download;
use dynamism::scraper::parse;
use dynamism::segmentation::model::{EmbeddingResponse, EmbeddingTask};
use dynamism::segmentation::{pure::*, segment_pipe_start};
use dynamism::umap::umap;
use dynamism::websearch::search;
use tempfile::tempdir;
use tokio::sync::mpsc::channel;
use tokio::task::JoinSet;
#[tokio::test(flavor = "multi_thread")]
async fn init_pipe() -> Result<()> {
    let query = "ladder recursive training";

    let mut set = JoinSet::new();
    let results = search(query).await?;
    let response = download(results).await?;

    for (s, u) in response {
        let s = s.text().await.unwrap();
        let u = u.to_string();
        set.spawn(async move { parse((s, u)).await });
    }
    let mut parse: Vec<(String, String)> = Vec::new();
    while let Some(res) = set.join_next().await {
        parse.push(res??);
    }

    let task = parse.iter().map(|s| EmbeddingTask {
        source_text: s.0.to_string(),
        url: s.1.to_string(),
    });

    let (tel, tel_handle) = dynamism::telemetry::spawn();
    let model = load_model().await?;
    let (tx, rx) = channel(10);
    for t in task {
        let tx = tx.clone();
        let model = model.clone();
        tokio::spawn(async move {
            let EmbeddingTask { source_text, url } = t;
            let chunks = segment_pipe_start(&source_text, &url, model).await.unwrap();
            tx.send(Ok(EmbeddingResponse { chunks })).await.unwrap();
        });
    }
    drop(tx);
    let fitted_chunks = umap(rx, tel.clone()).await?;
    let dir = tempdir()?;
    let db_handle = spawn(
        fitted_chunks,
        dir.path().to_str().unwrap().to_string(),
        "test".to_string(),
    );
    drop(tel);
    db_handle.await.unwrap();
    tel_handle.await.unwrap();
    Ok(())
}
