use anyhow::Result;

use tokio;
pub mod db;
pub mod embed;
pub mod readability;
pub mod reqwest;
pub mod scraper;
pub mod websearch;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
