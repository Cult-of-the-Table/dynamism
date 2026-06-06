use crate::segmentation::EmbeddedChunk;

pub struct EmbeddingTask {
    pub source_text: String,
    pub url: String,
}

pub struct EmbeddingResponse {
    pub chunks: Vec<EmbeddedChunk>,
}
