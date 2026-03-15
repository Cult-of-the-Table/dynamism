use fastembed::{Embedding, EmbeddingModel, Error, InitOptions, TextEmbedding};

pub fn embd(data: &[String]) -> Result<Vec<Embedding>, Error> {
    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::NomicEmbedTextV15).with_show_download_progress(true),
    )?;
    let embeddings = model.embed(data, None).map(|s| {
        println!("Embeddings length: {}", s.len());
        s
    });
    embeddings
}
