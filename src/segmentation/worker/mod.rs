use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use std::sync::mpsc::{Receiver, Sender, channel};

use crate::segmentation::*;
use model::*;

pub mod model;

pub async fn work(
    task: EmbeddingTask,
    sigma: f64,
    model: &mut TextEmbedding,
) -> Result<EmbeddingResponse> {
    let EmbeddingTask { source_text, url } = task;

    Ok(EmbeddingResponse {
        chunks: chunker(&source_text, &url, sigma, model).await?,
    })
}

pub fn spawn() -> (Sender<EmbeddingTask>, Receiver<Result<EmbeddingResponse>>) {
    let (tx, _rx) = channel();
    let (_tx, rx) = channel();

    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();

    std::thread::spawn(async move || {
        loop {
            _tx.send(
                work(
                    _rx.recv().expect("Embedding task should receive"),
                    0.1,
                    &mut model,
                )
                .await,
            )
            .expect("Embedding response should send");
        }
    });

    (tx, rx)
}
