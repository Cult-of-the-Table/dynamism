use anyhow::Result;
use main::reqwest::download;
use main::scraper::parse;
//use main::segmenter::segment;
use main::websearch::search;
use main::segmentation::segment;
#[tokio::test]
async fn init() -> Result<()> {
    let query = "rust language";

    let results = search(query).await.unwrap();
    let response = download(results).await.unwrap();
    let parse = parse(response).await.unwrap();
    let sentences = segment()

    // let sentences = segment(parse).unwrap();
    sentences.iter().for_each(|s| {
        s.iter().for_each(|a| println!("test: {}", a));
    });
    Ok(())
}
