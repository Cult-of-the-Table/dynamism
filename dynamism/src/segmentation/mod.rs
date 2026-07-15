use anyhow::Error;
use fastembed::Embedding;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::future::try_join_all;
use model::EmbeddedChunk;
use pure::*;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::{self, sync};

use crate::telemetry::TelEvent;
pub mod model;
pub mod pure;
//pub mod worker;

pub struct Batch {
    text: String,
    reply: sync::oneshot::Sender<Embedding>,
}
pub async fn segment_pipe_start(
    source: &str,
    url: &str,
    e_tx: Sender<Batch>,
) -> Result<Vec<EmbeddedChunk>, Error> {
    let pipeline = crate::Pipeline::inject(source);
    let (segments, ranges) = pipeline.segment().value?;
    let source = Arc::new(source.to_string());
    let url = Arc::new(url.to_string());
    let mut receivers = Vec::with_capacity(segments.len());
    // packages the segments with a receiveer and passes them to the embedding model
    for seg in segments {
        let (r_tx, r_rx) = tokio::sync::oneshot::channel();
        let msg = Batch {
            text: seg,
            reply: r_tx,
        };
        e_tx.send(msg).await?;
        receivers.push(r_rx);
    }
    let embeds = try_join_all(receivers).await.unwrap();
    let pipeline = crate::Pipeline::inject(AssemblyInput {
        embeds,
        ranges,
        source,
        url,
    })
    .assemble_chunks()
    .merge_chunks(0.1);
    Ok(pipeline.value)
}
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
                    let _ = msg.reply.send(emb);
                }
            } else {
                buff.clear()
            }
        }
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[tokio::test]
    async fn seg_test() {
        let text = "Hello world. This is Rust.";
        let sentences = segment(text).unwrap();
        let segments = sentences
            .iter()
            .map(|&Range { start, end }| &text[start..end])
            .collect::<Vec<&str>>();
        assert_eq!(segments, &["Hello world. ", "This is Rust."])
    }
}
