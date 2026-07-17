use anyhow::Result;
use dynamism::db::worker::spawn;
use dynamism::reqwest::download;
use dynamism::scraper::parse;
use dynamism::segmentation::model::EmbeddingResponse;
use dynamism::segmentation::model::EmbeddingTask;
use dynamism::umap::FittedChunks;
use dynamism::umap::umap;
use dynamism::websearch::search;
use futures::StreamExt;
use lancedb::{
    self,
    query::{ExecutableQuery, QueryBase},
};
use serde_arrow;
use std::env;
use std::path::PathBuf;
use tokio::sync::mpsc::channel;
//use tempfile::tempdir;
use tokio::task::JoinSet;
pub async fn read(dir: String) -> Vec<FittedChunks> {
    let db = lancedb::connect(&dir).execute().await.unwrap();
    let table = db.open_table("embeds").execute().await.unwrap();
    let mut stream = table
        .query()
        .select(lancedb::query::Select::All)
        .execute()
        .await
        .unwrap();
    let mut chunks = Vec::new();
    while let Some(result) = stream.next().await {
        let batch = result.unwrap();
        let records: Vec<FittedChunks> = serde_arrow::from_record_batch(&batch).unwrap();
        chunks.extend(records);
    }
    chunks
}
pub async fn load(query: String) -> Result<()> {
    let mut set = JoinSet::new();
    let results = search(query.as_str()).await?;
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
    let (batch_tx, batch_rx) = channel(100);
    let embed_handle = dynamism::Pipeline::inject(batch_rx)
        .embed_loop()
        .await
        .value;

    //let (b_tx, b_rx) = tokio::sync::oneshot::channel();
    //    let _ = tel
    //        .send(TelEvent::CreateBar {
    //            total: 0,
    //            style: ProgressStyle::default_bar(),
    //            reply: b_tx,
    //        })
    //        .await;
    //let bar_reply = b_rx.await.unwrap();
    let (tx, rx) = channel(10);
    for t in task {
        let batch_tx = batch_tx.clone();
        let tx = tx.clone();
        //let bar_tx = bar_reply.clone();
        tokio::spawn(async move {
            let EmbeddingTask { source_text, url } = t;
            let chunks = dynamism::segmentation::segment_pipe_start(&source_text, &url, batch_tx)
                .await
                .unwrap();
            tx.send(Ok(EmbeddingResponse { chunks })).await.unwrap();
        });
    }
    drop(batch_tx);
    drop(tx);
    let fitted_chunks = umap(rx, tel.clone()).await?;
    let mut dir: PathBuf = env::current_dir().unwrap();
    dir.push("db/");
    let db_handle = spawn(
        fitted_chunks,
        dir.as_path().to_str().unwrap().to_string(),
        "embeds".to_string(),
    );
    drop(tel);
    db_handle.await.unwrap();
    embed_handle.await.unwrap();
    tel_handle.await.unwrap();
    Ok(())
}
