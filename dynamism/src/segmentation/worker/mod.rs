use anyhow::Result;
use candle_core::{DType, Device};
use fastembed::NomicV2MoeTextEmbedding;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

use crate::segmentation::*;
use crate::telemetry::{BarEvent, TelEvent};
use model::*;

pub mod model;

pub async fn work(
    task: EmbeddingTask,
    sigma: f64,
    e_tx: Sender<Batch>,
    bar_tx: Sender<BarEvent>,
) -> Result<EmbeddingResponse> {
    //println!("work started");
    let EmbeddingTask { source_text, url } = task;

    Ok(EmbeddingResponse {
        chunks: chunker(&source_text, &url, sigma, e_tx.clone(), bar_tx.clone()).await?,
    })
}

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

    //let device = Device::Cpu;
    //let model = NomicV2MoeTextEmbedding::from_hf(
    //    "nomic-ai/nomic-embed-text-v2-moe",
    //    &device,
    //    DType::F32,
    //    768,
    //)
    //.unwrap();
    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )
    .unwrap();
    //   println!("model loaded");
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
    tokio::spawn(async move {
        //      println!("embed started");
        let mut buff: Vec<Batch> = Vec::new();
        while e_rx.recv_many(&mut buff, 1).await > 0 {
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
                    let _ = msg.reply.send(embedding);
                }
            } else {
                buff.clear()
            }
        }
        let _ = emb_bar_reply
            .send(BarEvent::Finish("Embedding Complete".to_string()))
            .await;
    });

    let handle = tokio::spawn(async move {
        //     println!("thread spawned");
        while let Some(msg) = _rx.recv().await {
            let _tx = _tx.clone();
            let e_tx = e_tx.clone();
            let bar_tx = bar_reply.clone();
            let tel = tel.clone();
            tokio::spawn(async move {
                let (s_tx, s_rx) = tokio::sync::oneshot::channel();
                let _ = tel.send(TelEvent::CreateSpinner { reply: s_tx }).await;
                let reply = s_rx.await.unwrap();
                _tx.send(work(msg, 0.1, e_tx, bar_tx).await).await.unwrap();
                let _ = reply.send(BarEvent::Finish("Done".to_string())).await;
            });
        }
        drop(e_tx);
        drop(tel);
    });

    (tx, rx, handle)
}
