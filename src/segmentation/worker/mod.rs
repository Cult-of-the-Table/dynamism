use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

use crate::segmentation::*;
use crate::telemetry::TelEvent;
use model::*;

pub mod model;

pub async fn work(
    task: EmbeddingTask,
    sigma: f64,
    e_tx: Sender<Batch>,
    tel: Sender<TelEvent>,
) -> Result<EmbeddingResponse> {
    //println!("work started");
    let EmbeddingTask { source_text, url } = task;

    Ok(EmbeddingResponse {
        chunks: chunker(&source_text, &url, sigma, e_tx.clone(), tel.clone()).await?,
    })
}

pub fn spawn(
    tel: Sender<TelEvent>,
) -> (
    Sender<EmbeddingTask>,
    Receiver<Result<EmbeddingResponse>>,
    JoinHandle<()>,
) {
    println!("Spawn start");
    let (tx, mut _rx) = channel(10);
    let (_tx, rx) = channel(10);
    let (e_tx, mut e_rx) = channel(100);

    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();
    println!("model loaded");
    let tel2 = tel.clone();
    tokio::spawn(async move {
        println!("embed started");
        let mut buff: Vec<Batch> = Vec::new();
        while e_rx.recv_many(&mut buff, 1).await > 0 {
            let text = buff.iter().map(|s| s.text.as_str()).collect::<Vec<&str>>();
            if let Ok(embedding) = model.embed(text, None) {
                tel2.send(TelEvent::ProcessedData(buff.len() as u64))
                    .await
                    .unwrap();
                for (msg, embedding) in buff.drain(..).zip(embedding) {
                    let _ = msg.reply.send(embedding);
                }
            } else {
                buff.clear()
            }
        }
    });

    let handle = tokio::spawn(async move {
        println!("thread spawned");
        while let Some(msg) = _rx.recv().await {
            let _tx = _tx.clone();
            let e_tx = e_tx.clone();
            let tel = tel.clone();
            tokio::spawn(async move {
                _tx.send(work(msg, 0.1, e_tx, tel).await).await.unwrap();
            });
        }
        drop(tel);
    });

    (tx, rx, handle)
}
