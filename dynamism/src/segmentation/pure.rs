use anyhow::Error;
use fastembed::{Embedding, EmbeddingModel, InitOptions, TextEmbedding};
use icu_segmenter::{SentenceSegmenter, options::SentenceBreakInvariantOptions};
use itertools::Itertools;
use std::ops::Range;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::Pipeline;
use crate::segmentation::model::EmbeddedChunk;

pub fn cosine_similarity(a: &Embedding, b: &Embedding) -> f64 {
    let a: &[f32] = a;
    let b: &[f32] = b;
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as f64) * (*y as f64))
        .sum();
    let mag_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}
pub struct AssemblyInput {
    pub ranges: Vec<Range<usize>>,
    pub embeds: Vec<Embedding>,
    pub source: Arc<String>,
    pub url: Arc<String>,
}
pub struct EmbedRequest {
    pub text: Vec<String>,
    pub reply: oneshot::Sender<Result<Vec<Embedding>, Error>>,
}

pub async fn load_model() -> Result<Sender<EmbedRequest>, Error> {
    let (tx, mut rx) = mpsc::channel::<EmbedRequest>(32);
    let (init_tx, init_rx) = oneshot::channel();
    std::thread::spawn(move || {
        let mut model = match TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
        ) {
            Ok(m) => {
                let _ = init_tx.send(Ok(()));
                m
            }
            Err(e) => {
                let _ = init_tx.send(Err(e));
                return;
            }
        };
        while let Some(req) = rx.blocking_recv() {
            let prefixed: Vec<String> = req
                .text
                .iter()
                .map(|s| format!("search_document: {s}"))
                .collect();
            let _ = req.reply.send(model.embed(prefixed, Some(32)));
        }
    });
    init_rx.await??;
    Ok(tx)
}
impl Pipeline<Sender<EmbedRequest>> {
    pub async fn embed(self, text: Vec<String>) -> Pipeline<Result<Vec<Embedding>, Error>> {
        self.map_async("Embedding segments...", move |tx| async move {
            let (reply_tx, reply_rx) = oneshot::channel();
            tx.send(EmbedRequest {
                text,
                reply: reply_tx,
            })
            .await?;
            reply_rx.await?
        })
        .await
    }
}
impl Pipeline<&str> {
    pub fn segment(self) -> Pipeline<Result<(Vec<String>, Vec<Range<usize>>), Error>> {
        self.map("Segmenting text...", move |text| {
            let segmenter = SentenceSegmenter::new(SentenceBreakInvariantOptions::default());
            let mut ranges = segmenter
                .segment_str(text)
                .tuple_windows()
                .map(|(i, j)| i..j)
                .collect::<Vec<Range<usize>>>();
            if ranges.len() > 700 {
                ranges.truncate(700);
            }
            let segments = ranges
                .iter()
                .map(|&Range { start, end }| text[start..end].to_string())
                .collect::<Vec<String>>();
            Ok((segments, ranges))
        })
    }
}

impl Pipeline<Result<AssemblyInput, Error>> {
    pub fn assemble_chunks(self) -> Pipeline<Result<Vec<EmbeddedChunk>, Error>> {
        self.map("Assembling chunk types...", |res| {
            res.map(|input| {
                input
                    .ranges
                    .into_iter()
                    .zip(input.embeds)
                    .map(|(range, embedding)| EmbeddedChunk {
                        id: Uuid::new_v4(),
                        source_url: input.url.clone(),
                        source_text: input.source.clone(),
                        range,
                        embedding,
                    })
                    .collect::<Vec<EmbeddedChunk>>()
            })
        })
    }
}
impl Pipeline<Result<Vec<EmbeddedChunk>, Error>> {
    pub fn merge_chunks(self, sigma: f64) -> Pipeline<Result<Vec<EmbeddedChunk>, Error>> {
        self.map("Merging vectors...", |res| {
            res.map(|chunks| {
                if chunks.is_empty() {
                    return vec![];
                };
                let mut merged: Vec<EmbeddedChunk> = vec![chunks[0].clone()];
                for window in chunks.windows(2) {
                    let prev = &window[0];
                    let curr = &window[1];
                    let sim = cosine_similarity(&prev.embedding, &curr.embedding);
                    if sim > 1.0 - sigma {
                        let last = merged.last_mut().unwrap();
                        last.range = last.range.start..curr.range.end;
                    } else {
                        merged.push(curr.clone());
                    }
                }
                merged
            })
        })
    }
}
