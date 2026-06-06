use anyhow::{Error, Result};
use reqwest::Response;
use tokio::task::JoinSet;
use websearch::SearchResult;
pub async fn download(urls: Vec<SearchResult>) -> Result<Vec<(Response, String)>, Error> {
    let mut set = JoinSet::new();
    urls.into_iter().for_each(|s| {
        set.spawn(async move { (reqwest::get(s.url.to_owned()).await, s.url.to_owned()) });
    });
    let mut download: Vec<(Response, String)> = Vec::new();
    while let Some(res) = set.join_next().await {
        match res {
            Ok((Ok(res), url)) => download.push((res, url.to_string())),
            Ok((Err(e), url)) => {
                println!("Error: {}, Failed to get url {}", e, url)
            }
            Err(e) => {
                println!("Error: {}", e)
            }
        }
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
            title: "Rust".to_string(),
            snippet: None,
            domain: None,
            published_date: None,
            provider: None,
            raw: None,
        };
        let response = download(vec![search]).await?;
        assert_eq!(response.len(), 1);
        Ok(())
    }
}
