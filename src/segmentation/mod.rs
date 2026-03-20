use anyhow::Error;
use fastembed::Embedding;
use icu_segmenter::{options::SentenceBreakInvariantOptions, SentenceSegmenter};
use itertools::Itertools;
use uuid::Uuid;

use crate::embedding::embd;
use model::{Chunk, EmbeddedChunk};

pub mod model;

fn cosine_similarity(a: &Embedding, b: &Embedding) -> f64 {
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

pub async fn segment(s: &str, source_url: &str, sigma: f64) -> Result<Vec<EmbeddedChunk>, Error> {
    let segmenter = SentenceSegmenter::new(SentenceBreakInvariantOptions::default());
    let chunks: Vec<Chunk> = segmenter
        .segment_str(s)
        .tuple_windows()
        .map(|(i, j)| Chunk {
            id: Uuid::new_v4(),
            source_url: source_url.to_string(),
            source_text: s.to_string(),
            range: i..j,
        })
        .collect();

    let texts: Vec<String> = chunks.iter().map(|c| c.chunk_text().to_string()).collect();
    let embeddings = embd(&texts).await?;

    let embedded_chunks: Vec<EmbeddedChunk> = chunks
        .into_iter()
        .zip(embeddings.into_iter())
        .map(|(chunk, embedding)| EmbeddedChunk {
            id: chunk.id,
            source_url: chunk.source_url,
            source_text: chunk.source_text,
            range: chunk.range,
            embedding,
        })
        .collect();

    if embedded_chunks.is_empty() {
        return Ok(vec![]);
    }

    let mut merged: Vec<EmbeddedChunk> = vec![embedded_chunks[0].clone()];

    for window in embedded_chunks.windows(2) {
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

    Ok(merged)
}
