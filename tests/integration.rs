use anyhow::Result;
use main::reqwest::download;
use main::scraper::parse;
use tokio::task::JoinSet;
//use main::segmenter::segment;
use main::segmentation::{model::EmbeddedChunk, segment};
use main::websearch::search;
#[tokio::test]
async fn init() -> Result<()> {
    let query = "rust language";

    let results = search(query).await?;
    let response = download(results).await?;
    let parse = parse(response).await?;
    let mut set = JoinSet::new();
    parse.into_iter().for_each(|s| {
        set.spawn(async move {
            let (s, u) = s;
            segment(s.as_str(), u.as_str(), 0.5).await.unwrap()
        });
    });
    let mut chunks: Vec<Vec<EmbeddedChunk>> = Vec::new();
    while let Some(res) = set.join_next().await {
        let s = res?;
        chunks.push(s);
    }
    //sentences.iter().for_each(|s| {
    //    s.iter().for_each(|a| println!("test: {}", a));
    //});
    Ok(())
}
