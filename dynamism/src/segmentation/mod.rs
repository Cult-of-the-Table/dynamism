use anyhow::Error;
use model::EmbeddedChunk;
use pure::*;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use crate::Pipeline;

pub mod model;
pub mod pure;

pub async fn segment_pipe_start(
    source: &str,
    url: &str,
    model: Sender<EmbedRequest>,
    query: &str,
) -> Result<Vec<EmbeddedChunk>, Error> {
    let pipeline = crate::Pipeline::inject(source);
    let (segments, ranges) = pipeline.segment().value?;
    let source = Arc::new(source.to_string());
    let url = Arc::new(url.to_string());
    let embeds = Pipeline::inject(&model)
        .embed(segments)
        .await
        .query_filter(&model, &query)
        .await
        .value?;
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
