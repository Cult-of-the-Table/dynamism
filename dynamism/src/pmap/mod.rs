use crate::segmentation::model::EmbeddingResponse;
use crate::telemetry::TelEvent;
use ndarray::Array2;
use pacmap::{Configuration, fit_transform};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Clone, serde::Deserialize, Debug, Default)]
pub struct FittedChunks {
    pub url: Arc<String>,
    pub snippet: Arc<String>,
    pub text: Arc<String>,
    pub embeds: Coords,
}
#[derive(Copy, serde::Deserialize, Clone, Debug, Default)]
#[serde(from = "[f32; 2]")]
pub struct Coords {
    pub x: f32,
    pub y: f32,
}
impl From<Coords> for [f32; 2] {
    fn from(c: Coords) -> Self {
        [c.x, c.y]
    }
}
impl From<[f32; 2]> for Coords {
    fn from(v: [f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}
use anyhow::Result;
// @todo(2026-06-18): make umap stream results instead of bulk data processing.
pub async fn pmap(
    mut _rx: Receiver<Result<EmbeddingResponse>>,
    _tel: Sender<TelEvent>,
) -> Result<Vec<FittedChunks>> {
    let (mut u, mut t, mut ct, mut c) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    while let Some(Ok(EmbeddingResponse { chunks })) = _rx.recv().await {
        chunks.into_iter().for_each(|s| {
            u.push(s.source_url.clone());
            t.push(s.source_text.clone());
            ct.push(Arc::new(s.chunk_text().to_string()));
            c.push(s.embedding);
        })
    }
    let data = Array2::from_shape_vec((c.len(), 768), c.into_iter().flatten().collect())?;
    let config = Configuration::default();
    let (fitted, _) = fit_transform(data.view(), config)?;
    let fitted_chunks = fitted
        .rows()
        .into_iter()
        .zip(u)
        .zip(t)
        .zip(ct)
        .map(|(((embeds, url), _text), snippet)| {
            let coords = Coords {
                x: embeds[0],
                y: embeds[1],
            };

            FittedChunks {
                url,
                snippet,
                text: Arc::new("".to_string()),
                embeds: coords,
            }
        })
        .collect::<Vec<FittedChunks>>();
    Ok(fitted_chunks)
}
