use fastembed::{Embedding, EmbeddingModel, Error, InitOptions, TextEmbedding};

pub fn embd(data: Vec<&str>) -> Result<Vec<Embedding>, Error> {
    let data = data.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )?;
    let embeddings = model.embed(data, None).map(|s| {
        println!("Embeddings length: {}", s.len());
        s
    });
    embeddings
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init() {
        let v = vec!["test", "test2"];
        let _ = embd(v);
    }
}
