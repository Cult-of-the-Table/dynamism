use anyhow::Result;
use scraper::Html;

pub async fn parse(html: (String, String)) -> Result<(String, String)> {
    let download = Some(html).map(|(s, u)| (Html::parse_document(s.as_str()), u));
    let text = Some(download.unwrap()).map(|(s, u)| {
        (
            s.root_element()
                .text()
                .flat_map(|s| s.split_whitespace())
                .collect::<Vec<_>>()
                .join(" "),
            u,
        )
    });
    Ok(text.unwrap())
}
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::reqwest::download;
    use tokio::task::JoinSet;
    use websearch::SearchResult;
    #[tokio::test]
    async fn scraper() -> Result<()> {
        let mut set = JoinSet::new();
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
        for (s, u) in response {
            let s = s.text().await.unwrap();
            let u = u.to_string();
            set.spawn(async move { parse((s, u)).await });
        }
        while let Some(res) = set.join_next().await {
            let (s, _) = res??;
            println!("Text: {}", s);
        }
        Ok(())
    }
}
