use anyhow::{Error, Result};
use reqwest::Response;
use tokio::task::JoinSet;
use websearch::SearchResult;
async fn download(urls: Vec<SearchResult>) -> Result<Vec<Response>, Error> {
    let mut set = JoinSet::new();
    urls.into_iter().for_each(|s| {
        set.spawn(async move { reqwest::get(s.url).await });
    });
    let mut download: Vec<Response> = Vec::new();
    while let Some(res) = set.join_next().await {
        let text = res??;
        download.push(text);
    }
    Ok(download)
}
#[cfg(test)]
pub mod tests {
    use super::*;
    #[tokio::test]
    async fn init() -> Result<()> {
        let search = SearchResult {
            url: ("https://rust-lang.org/").to_string(),
            title: "Rust".to_string(), // or String::new()
            snippet: None,
            domain: None,
            published_date: None,
            provider: None,
            raw: None,
        };
        let _ = download(vec![search]);
        Ok(())
    }
}
