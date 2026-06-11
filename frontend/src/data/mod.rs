use anyhow::Result;
use dynamism::db::worker::spawn;
use dynamism::reqwest::download;
use dynamism::scraper::parse;
use dynamism::segmentation::worker::model::EmbeddingTask;
use dynamism::umap::umap;
use dynamism::websearch::search;
use std::env;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::task::JoinSet;
pub async fn load(query: String) -> Result<()> {
    let mut set = JoinSet::new();
    let results = search(query.as_str()).await?;
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

    let (tel, tel_handle) = dynamism::telemetry::spawn();
    let (tx, rx, seg_handle) = dynamism::segmentation::worker::spawn(tel.clone());
    tx.send(task).await.unwrap();
    drop(tx);

    let fitted_chunks = umap(rx).await?;
    //let dir = tempdir()?;
    let mut dir: PathBuf = env::current_dir().unwrap();
    dir.push("db/");
    let db_handle = spawn(
        fitted_chunks,
        dir.as_path().to_str().unwrap().to_string(),
        "test".to_string(),
    );
    drop(tel);
    db_handle.await.unwrap();
    seg_handle.await.unwrap();
    tel_handle.await.unwrap();
    Ok(())
}
