use anyhow::Error;
use fastembed::Embedding;
use futures::future::try_join_all;
use model::{Batch, EmbeddedChunk};
use pure::*;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::{self, sync};

pub mod model;
pub mod pure;

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
    let pipeline = crate::Pipeline::inject(Ok(AssemblyInput {
        embeds,
        ranges,
        source,
        url,
    }))
    .assemble_chunks()
    .merge_chunks(0.1);
    Ok(pipeline.value?)
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
