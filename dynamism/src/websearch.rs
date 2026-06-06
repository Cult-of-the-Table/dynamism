use anyhow::Result;
use websearch::{
    SearchError, SearchResult,
    multi_provider::{
        MultiProviderConfig, MultiProviderSearch, MultiProviderStrategy, SearchOptionsMulti,
    },
    providers::{ArxivProvider, DuckDuckGoProvider},
};

pub async fn search(query: &str) -> Result<Vec<SearchResult>, SearchError> {
    let strategy = MultiProviderStrategy::Aggregate;
    let schema = MultiProviderConfig::new(strategy);
    let schema = schema.add_provider(Box::new(ArxivProvider::new()));
    let schema = schema.add_provider(Box::new(DuckDuckGoProvider::new()));
    let mut multi = MultiProviderSearch::new(schema);
    let search = SearchOptionsMulti {
        query: query.to_string(),
        ..Default::default()
    };
    let results = multi.search(&search).await;
    results.iter().for_each(|s| {
        s.iter().for_each(|v| {
            println! {"{}",v.title};
        })
    });
    results
}
#[cfg(test)]
pub mod tests {
    use super::*;
    #[tokio::test]
    async fn init() -> Result<()> {
        let query = "rust";
        let _ = search(query).await;
        Ok(())
    }
}
