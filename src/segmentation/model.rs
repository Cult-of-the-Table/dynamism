use std::collections::HashMap;
use std::ops::Range;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use fastembed::Embedding;

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    pub id: Uuid,
    pub source_url: String,
    pub source_text: String,
    pub range: Range<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct EmbeddedChunk {
    pub id: Uuid,
    pub source_url: String,
    pub source_text: String,
    pub range: Range<usize>,
    pub embedding: Embedding,
}

impl Chunk {
    pub fn chunk_text(&self) -> &str {
        &self.source_text[self.range.clone()]
    }
}

impl EmbeddedChunk {
    pub fn chunk_text(&self) -> &str {
        &self.source_text[self.range.clone()]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Point3 {
    pub id: Uuid,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Point3 {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UmapState {
    pub conversation_id: String,
    pub chunk_count: usize,
    pub id_to_index: HashMap<Uuid, usize>,
    pub embeddings: Vec<Vec<f64>>,
    pub points: Vec<Point3>,
    pub novelty_rate: f64,
    pub density_drift: f64,
    pub model_bytes: Option<Vec<u8>>,
}

impl Default for UmapState {
    fn default() -> Self {
        Self {
            conversation_id: String::new(),
            chunk_count: 0,
            id_to_index: HashMap::new(),
            embeddings: Vec::new(),
            points: Vec::new(),
            novelty_rate: 0.0,
            density_drift: 0.0,
            model_bytes: None,
        }
    }
}
