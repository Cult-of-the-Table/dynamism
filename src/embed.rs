use fastembed::{Embedding, EmbeddingModel, Error, InitOptions, TextEmbedding};

pub async fn embd(data: Vec<String>) -> Result<Vec<Embedding>, Error> {
    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )?;
    model.embed(data, None).inspect(|s| {
        println!("Embeddings length: {}", s.len());
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init() {
        let v = vec!["test", "test2"];
        let _ = embd(v.iter().map(|s| s.to_string()).collect::<Vec<String>>());
    }
}
