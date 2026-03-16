use anyhow::Result;
use reqwest::Response;
use scraper::Html;
use tokio::task::JoinSet;

pub async fn parse(html: Vec<Response>) -> Result<Vec<Html>> {
    let mut set = JoinSet::new();
    html.into_iter().for_each(|s| {
        set.spawn(async move { s.text().await.unwrap() });
    });
    let mut downloads: Vec<Html> = Vec::new();
    while let Some(res) = set.join_next().await {
        downloads.push(Html::parse_document(res.unwrap().as_str()));
    }
    Ok(downloads)
}
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::reqwest::download;
    use websearch::SearchResult;
    #[tokio::test]
    async fn init() -> Result<()> {
        let search = SearchResult {
            url: "https://rust-lang.org/".to_string(),
            title: "Rust".to_string(),
            snippet: None,
            domain: None,
            published_date: None,
            provider: None,
            raw: None,
        };
        let response = download(vec![search]).await.unwrap();
        let parse = parse(response).await?;
        let text = parse
            .into_iter()
            .map(|s| {
                let text = s
                    .root_element()
                    .text()
                    .flat_map(|v| v.split_whitespace())
                    .collect::<Vec<_>>()
                    .join(" ");
                text
            })
            .collect::<Vec<String>>();
        text.iter().for_each(|s| println!("Text: {}", s));
        Ok(())
    }
}
