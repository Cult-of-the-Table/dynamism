use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use indicatif::ProgressStyle;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

use crate::segmentation::*;
use crate::telemetry::{BarEvent, TelEvent};
use model::*;
pub mod model;

pub async fn embed_loop(mut rx: Receiver<Batch>, tel: Sender<TelEvent>) -> JoinHandle<()> {
    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();
    tokio::spawn(async move {
        let mut buff: Vec<Batch> = Vec::new();
        while rx.recv_many(&mut buff, 1).await > 0 {
            let text = buff
                .iter()
                .map(|s| format!("search_document: {}", s.text))
                .collect::<Vec<_>>();
            let text = text.iter().map(|s| s.as_str()).collect::<Vec<_>>();
            if let Ok(embedding) = model.embed(text, None) {
                for (msg, emb) in buff.drain(..).zip(embedding) {
                    let _ = msg.reply.send(emb); // sent to segmentation::chunk()
                }
            } else {
                buff.clear()
            }
        }
    })
}
#[deprecated]
pub async fn spawn(
    tel: Sender<TelEvent>,
) -> (
    Sender<EmbeddingTask>,
    Receiver<Result<EmbeddingResponse>>,
    JoinHandle<()>,
) {
    //    println!("Spawn start");
    let (tx, mut _rx) = channel(10);
    let (_tx, rx) = channel(10);
    let (e_tx, mut e_rx) = channel(100);

    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();
    let (b_tx, b_rx) = tokio::sync::oneshot::channel();
    let _ = tel
        .send(TelEvent::CreateBar {
            total: 0,
            style: ProgressStyle::default_bar(),
            reply: b_tx,
        })
        .await;
    let bar_reply = b_rx.await.unwrap();
    let emb_bar_reply = bar_reply.clone();

    // receives data from `super::chunker()` and passes it into the embedding model
    tokio::spawn(async move {
        let mut buff: Vec<Batch> = Vec::new();
        while e_rx.recv_many(&mut buff, 1).await > 0 {
            // rec from segmentation::chunk()
            let text = buff
                .iter()
                .map(|s| format!("search_document: {}", s.text))
                .collect::<Vec<String>>();
            let text = text.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            if let Ok(embedding) = model.embed(text, None) {
                emb_bar_reply
                    .send(BarEvent::Inc(buff.len() as u64))
                    .await
                    .unwrap();
                for (msg, embedding) in buff.drain(..).zip(embedding) {
                    let _ = msg.reply.send(embedding); // sent to segmentation::chunk()
                }
            } else {
                buff.clear()
            }
        }
        let _ = emb_bar_reply
            .send(BarEvent::Finish("Embedding Complete".to_string()))
            .await;
    });

    // populates the eventually returned rx channel with the result of `work()`
    let handle = tokio::spawn(async move {
        while let Some(msg) = _rx.recv().await {
            // rec from db::load()

            let _tx = _tx.clone();
            let e_tx = e_tx.clone();

            // telemetry items:
            let bar_tx = bar_reply.clone();
            let tel = tel.clone();

            tokio::spawn(async move {
                let (s_tx, s_rx) = tokio::sync::oneshot::channel();
                let _ = tel.send(TelEvent::CreateSpinner { reply: s_tx }).await;
                let reply = s_rx.await.unwrap();
                let EmbeddingTask { source_text, url } = msg;
                let response = EmbeddingResponse {
                    chunks: chunk(
                        segment(&source_text).await.unwrap(),
                        &url,
                        &source_text,
                        0.1,
                        e_tx.clone(),
                        bar_tx.clone(),
                    )
                    .await
                    .unwrap(),
                };
                _tx.send(Ok(response)).await.unwrap(); // sent to umap::umap()
                let _ = reply.send(BarEvent::Finish("Done".to_string())).await;
            });
        }
        drop(e_tx);
        drop(tel);
    });

    (tx, rx, handle)
}
