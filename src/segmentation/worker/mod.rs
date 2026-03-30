use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

//use std::sync::mpsc::{Receiver, Sender, channel};

use crate::segmentation::*;
use crate::telemetry;
use crate::telemetry::TelEvent;
use model::*;

pub mod model;

pub async fn work(
    task: EmbeddingTask,
    sigma: f64,
    model: &mut TextEmbedding,
    tel: Sender<TelEvent>,
) -> Result<EmbeddingResponse> {
    println!("work started");
    let EmbeddingTask { source_text, url } = task;

    Ok(EmbeddingResponse {
        chunks: chunker(&source_text, &url, sigma, model, tel.clone()).await?,
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

    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();
    println!("model loaded");

    let handle = tokio::spawn(async move {
        println!("thread spawned");
        while let Some(msg) = _rx.recv().await {
            _tx.send(work(msg, 0.1, &mut model, tel.clone()).await)
                .await
                .unwrap();
        }
        drop(tel);
    });

    (tx, rx, handle)
}
