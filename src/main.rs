use anyhow::Result;

pub mod db;
pub mod embed;
pub mod reqwest;
pub mod scraper;
pub mod websearch;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
