use anyhow::Error;
use fastembed::{Embedding, TextEmbedding};
use icu_segmenter::{SentenceSegmenter, options::SentenceBreakInvariantOptions};
use itertools::Itertools;
use std::ops::Range;
use std::sync::Arc;
use uuid::Uuid;

use model::EmbeddedChunk;

pub mod model;
pub mod worker;

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

pub async fn chunker(
    source: &str,
    url: &str,
    sigma: f64,
    model: &mut TextEmbedding,
) -> Result<Vec<EmbeddedChunk>, Error> {
    let segment = segment(source).await.unwrap();
    chunk(segment, url, source, sigma, model).await
}
async fn chunk(
    ranges: Vec<Range<usize>>,
    url: &str,
    source: &str,
    sigma: f64,
    model: &mut TextEmbedding,
) -> Result<Vec<EmbeddedChunk>, Error> {
    println!("Chunk function start");
    let source = Arc::new(source.to_string());
    let url = Arc::new(url.to_string());
    let segments = ranges
        .iter()
        .map(|&Range { start, end }| source[start..end].to_string())
        .collect::<Vec<String>>();
    let embeds = model
        .embed(segments, None)
        .inspect(|s| println!("Embeddings length: {}", s.len()));

    let embedded_chunks = ranges
        .into_iter()
        .zip(embeds.into_iter().flatten())
        .map(|(range, embedding)| EmbeddedChunk {
            id: Uuid::new_v4(),
            source_url: url.clone(),
            source_text: source.clone(),
            range,
            embedding,
        })
        .collect::<Vec<EmbeddedChunk>>();

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
async fn segment(s: &str) -> Result<Vec<Range<usize>>, Error> {
    let segmenter = SentenceSegmenter::new(SentenceBreakInvariantOptions::default());
    let segments = segmenter
        .segment_str(s)
        .tuple_windows()
        .map(|(i, j)| i..j)
        .collect();
    Ok(segments)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[tokio::test]
    async fn seg_test() {
        let text = "Hello world. This is Rust.";
        let sentences = segment(text).await.unwrap();
        let segments = sentences
            .iter()
            .map(|&Range { start, end }| &text[start..end])
            .collect::<Vec<&str>>();
        assert_eq!(segments, &["Hello world. ", "This is Rust."])
    }
}
