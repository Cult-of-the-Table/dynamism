use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use model::*;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::segmentation::*;

pub mod model;

pub fn spawn() -> (Sender<EmbeddingTask>, Receiver<EmbeddingResponse>) {
    let (tx, _rx) = channel();
    let (_tx, rx) = channel();

    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();

    std::thread::spawn(async move || {
        loop {
            // acquire work
            match _rx.recv() {
                Ok(EmbeddingTask { source_text, url }) => {
                    _tx.send(EmbeddingResponse {
                        chunks: chunker(&source_text, &url, 0.1, &mut model)
                            .await
                            .expect("Embedding should succeed"),
                    })
                    .expect("Embedding response should send");
                }
                Err(E) => panic!(),
            }
        }
    });

    (tx, rx)
}
