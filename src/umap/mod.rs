use crate::segmentation::worker::model::EmbeddingResponse;
use crate::telemetry::{BarEvent, TelEvent};
use burn::backend::Autodiff;
use burn::backend::wgpu::CubeBackend;
use burn::backend::wgpu::WgpuRuntime;
use crossbeam_channel;
use fast_umap::{self, Umap, UmapConfig};
use indicatif::ProgressStyle;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Clone, Debug, Default)]
pub struct FittedChunks {
    pub url: Arc<String>,
    pub text: Arc<String>,
    pub embeds: Vec<f64>,
}
use anyhow::Result;
pub async fn umap(
    mut _rx: Receiver<Result<EmbeddingResponse>>,
    tel: Sender<TelEvent>,
) -> Result<Vec<FittedChunks>> {
    let config = UmapConfig {
        n_components: 3,
        ..Default::default()
    };
    let umap = Umap::<Autodiff<CubeBackend<WgpuRuntime, f32, i32, u32>>>::new(config.clone());

    let (mut u, mut t, mut c) = (Vec::new(), Vec::new(), Vec::new());
    while let Some(Ok(EmbeddingResponse { chunks })) = _rx.recv().await {
        chunks.into_iter().for_each(|s| {
            u.push(s.source_url.clone());
            t.push(s.source_text.clone());
            c.push(s.embedding);
        })
    }
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

    let (_exit_tx, exit_rx) = crossbeam_channel::bounded(1);
    let fitted =
        tokio::task::spawn_blocking(move || umap.fit_with_progress(c, None, exit_rx, progress))
            .await
            .unwrap();
    let _ = reply
        .send(BarEvent::Finish("Umap Complete".to_string()))
        .await;
    let new_embeds = fitted.embedding();
    let fitted_chunks = new_embeds
        .iter()
        .zip(u.into_iter())
        .zip(t.into_iter())
        .map(|((embeds, url), text)| FittedChunks {
            url,
            text,
            embeds: embeds.to_vec(),
        })
        .collect::<Vec<FittedChunks>>();

    Ok(fitted_chunks)
}
