use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

//use std::sync::mpsc::{Receiver, Sender, channel};

use crate::segmentation::*;
use model::*;

pub mod model;

pub async fn work(
    task: EmbeddingTask,
    sigma: f64,
    model: &mut TextEmbedding,
) -> Result<EmbeddingResponse> {
    println!("work started");
    let EmbeddingTask { source_text, url } = task;

    Ok(EmbeddingResponse {
        chunks: chunker(&source_text, &url, sigma, model).await?,
    })
}

pub fn spawn() -> (
    Sender<EmbeddingTask>,
    Receiver<Result<EmbeddingResponse>>,
    JoinHandle<()>,
) {
    println!("Spawn start");
    let (tx, mut _rx) = channel(10);
    let (_tx, rx) = channel(10);

    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();
    println!("model loaded");

    let handle = tokio::spawn(async move {
        println!("thread spawned");
        while let Some(msg) = _rx.recv().await {
            dbg!(_tx.send(work(msg, 0.1, &mut model).await).await).unwrap();
        }
    });

    (tx, rx, handle)
}
