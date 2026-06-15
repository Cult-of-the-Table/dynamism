use anyhow::Result;
use readability_rust::Readability;
use sanitize_html::rules::predefined::DEFAULT;
use sanitize_html::sanitize_str;
use scraper::Html;

pub async fn parse(html: (String, String)) -> Result<(String, String)> {
    let (text, url) = html;

    let sanitize_default: String = sanitize_str(&DEFAULT, &text).unwrap();
    let shtml = Html::parse_document(&sanitize_default.as_str());
    let mut parser = Readability::new_with_base_uri(&sanitize_default, &url, None).unwrap();

    if let Some(article) = parser.parse() {
        let stext = article
            .text_content
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        return Ok((stext, url));
    }
    let stext = shtml
        .root_element()
        .text()
        .flat_map(|s| s.split_whitespace())
        .collect::<Vec<_>>()
        .join(" ");
    Ok((stext, url))
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
