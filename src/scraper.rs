use anyhow::Result;
use reqwest::Response;
use scraper::Html;
use tokio::task::JoinSet;

pub async fn parse(html: Vec<(Response, String)>) -> Result<Vec<(String, String)>> {
    let mut set = JoinSet::new();
    html.into_iter().for_each(|s| {
        set.spawn(async move {
            let (s, u) = s;
            (s.text().await.unwrap(), u)
        });
    });
    let mut downloads: Vec<(Html, String)> = Vec::new();
    while let Some(res) = set.join_next().await {
        let (s, u) = res?;
        downloads.push((Html::parse_document(s.as_str()), u));
    }
    let text = downloads
        .into_iter()
        .map(|s| {
            let (s, u) = s;
            (
                s.root_element()
                    .text()
                    .flat_map(|v| v.split_whitespace())
                    .collect::<Vec<_>>()
                    .join(" "),
                u,
            )
        })
        .collect::<Vec<(String, String)>>();
    Ok(text)
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
        parse.iter().for_each(|s| {
            let (s, _) = s;
            println!("Text: {}", s);
        });
        Ok(())
    }
}
