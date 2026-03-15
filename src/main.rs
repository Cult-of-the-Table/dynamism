use anyhow::Result;
use tokio;
pub mod db;
pub mod embed;
pub mod websearch;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
