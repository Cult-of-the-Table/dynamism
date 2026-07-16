use anyhow::Error;
use fastembed::Embedding;
use fastembed::{Embedding, TextEmbedding};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use icu_segmenter::{SentenceSegmenter, options::SentenceBreakInvariantOptions};
use itertools::Itertools;
use std::ops::Range;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::model::Batch;
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

impl Pipeline<Receiver<Batch>> {
    pub async fn embed_loop(self) -> Pipeline<JoinHandle<()>> {
        self.run_with_logs("Embedding segments...", move |input| {
            let mut model = TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::NomicEmbedTextV15)
                    .with_show_download_progress(true),
            )
            .unwrap();
            tokio::spawn(async move {
                let mut buff: Vec<Batch> = Vec::new();
                while input.recv_many(&mut buff, 1).await > 0 {
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
        })
    }
}

impl Pipeline<&str> {
    pub fn segment(self) -> Pipeline<Result<(Vec<String>, Vec<Range<usize>>), Error>> {
        self.run_with_logs("Segmenting text...", move |text| {
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

impl Pipeline<AssemblyInput> {
    pub fn assemble_chunks(self) -> Pipeline<Vec<EmbeddedChunk>> {
        self.run_with_logs("Assembling chunk types...", move |input| {
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
    }
}
impl Pipeline<Vec<EmbeddedChunk>> {
    pub fn merge_chunks(self, sigma: f64) -> Pipeline<Vec<EmbeddedChunk>> {
        self.run_with_logs("Merging vectors...", move |chunks| {
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
    }
}
