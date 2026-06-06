use crate::segmentation::worker::model::EmbeddingResponse;
use burn::backend::Autodiff;
use burn::backend::wgpu::CubeBackend;
use burn::backend::wgpu::WgpuRuntime;
use fast_umap::{self, Umap, UmapConfig};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

#[derive(Clone, Debug, Default)]
pub struct FittedChunks {
    pub url: Arc<String>,
    pub text: Arc<String>,
    pub embeds: Vec<f64>,
}
use anyhow::Result;
pub async fn umap(mut _rx: Receiver<Result<EmbeddingResponse>>) -> Result<Vec<FittedChunks>> {
    let config = UmapConfig {
        n_components: 3,
        ..Default::default()
    };
    let umap = Umap::<Autodiff<CubeBackend<WgpuRuntime, f32, i32, u32>>>::new(config);

    let (mut u, mut t, mut c) = (Vec::new(), Vec::new(), Vec::new());
    while let Some(Ok(EmbeddingResponse { chunks })) = _rx.recv().await {
        chunks.into_iter().for_each(|s| {
            u.push(s.source_url.clone());
            t.push(s.source_text.clone());
            c.push(s.embedding);
        })
    }
    let fitted = umap.fit(c, None);
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
