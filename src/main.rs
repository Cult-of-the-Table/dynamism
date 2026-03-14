use anyhow::Result;
use tokio;
pub mod db;
#[tokio::main]
async fn main() -> anyhow::Result {
    println!("Hello, world!");
}
