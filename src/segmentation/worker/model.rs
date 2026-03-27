use std::ops::Range;
use std::sync::Arc;

pub struct EmbeddingTask {
    text: Arc<String>,
    range: Range<usize>,
}

pub struct EmbeddingResponse {
    text: Arc<String>,
    range: Range<usize>,
    embedding: Embedding,
}
