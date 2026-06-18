use crate::segmentation::worker::model::EmbeddingResponse;
use crate::telemetry::{BarEvent, TelEvent};
use burn::backend::Autodiff;
use burn::backend::wgpu::CubeBackend;
use burn::backend::wgpu::WgpuRuntime;
use crossbeam_channel;
use fast_umap::{self, GraphParams, ManifoldParams, Metric, OptimizationParams, Umap, UmapConfig};
use indicatif::ProgressStyle;
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
#[serde(from = "[f64; 2]")]
pub struct Coords {
    pub x: f64,
    pub y: f64,
}
impl From<Coords> for [f64; 2] {
    fn from(c: Coords) -> Self {
        [c.x, c.y]
    }
}
impl From<[f64; 2]> for Coords {
    fn from(v: [f64; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}
use anyhow::Result;
// @todo(2026-06-18): make umap stream results instead of bulk data processing.
pub async fn umap(
    mut _rx: Receiver<Result<EmbeddingResponse>>,
    tel: Sender<TelEvent>,
) -> Result<Vec<FittedChunks>> {
    // config
    let config = UmapConfig {
        n_components: 2,
        graph: GraphParams {
            n_neighbors: 8,
            metric: Metric::Cosine,
            ..Default::default()
        },
        optimization: OptimizationParams {
            n_epochs: 300,
            ..Default::default()
        },
        ..Default::default()
    };
    let umap = Umap::<Autodiff<CubeBackend<WgpuRuntime, f32, i32, u32>>>::new(config.clone());

    // rec from worker::spawn()
    let (mut u, mut t, mut ct, mut c) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    while let Some(Ok(EmbeddingResponse { chunks })) = _rx.recv().await {
        chunks.into_iter().for_each(|s| {
            u.push(s.source_url.clone());
            t.push(s.source_text.clone());
            ct.push(Arc::new(s.chunk_text().to_string()));
            c.push(s.embedding);
        })
    }

    // telemetry
    let (b_tx, b_rx) = tokio::sync::oneshot::channel();
    tel.send(TelEvent::CreateBar {
        total: config.optimization.n_epochs as u64,
        style: ProgressStyle::default_bar(),
        reply: b_tx,
    })
    .await?;
    let reply = b_rx.await?;
    let reply_prog = reply.clone();
    let progress = Box::new(move |progress: fast_umap::EpochProgress| {
        let _ = reply_prog.try_send(BarEvent::SetPos(progress.epoch as u64));
    });

    // chunks sent to umap
    let (_exit_tx, exit_rx) = crossbeam_channel::bounded(1);
    let fitted =
        tokio::task::spawn_blocking(move || umap.fit_with_progress(c, None, exit_rx, progress))
            .await
            .unwrap();
    let _ = reply
        .send(BarEvent::Finish("Umap Complete".to_string()))
        .await;

    // output of umap
    // zipped and collected into FittedChunks
    let new_embeds = fitted.embedding();
    let fitted_chunks = new_embeds
        .iter()
        .zip(u.into_iter())
        .zip(t.into_iter())
        .zip(ct.into_iter())
        .map(|(((embeds, url), _text), snippet)| {
            let mut iter = embeds.iter().map(|&s| s as f64);
            let coords = Coords {
                x: iter.next().unwrap_or(0.0),
                y: iter.next().unwrap_or(0.0),
            };

            FittedChunks {
                url,
                snippet,
                // @todo(2026-06-20): split db into text and embeddings
                text: Arc::new("".to_string()),
                embeds: coords,
            }
        })
        .collect::<Vec<FittedChunks>>();

    Ok(fitted_chunks)
}
