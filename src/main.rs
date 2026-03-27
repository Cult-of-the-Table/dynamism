use anyhow::Result;
use dynamism::{db, segmentation};
#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = segmentation::worker::spawn();
    db::worker::spawn(rx, "database".to_owned(), "dynamism_main_table".to_owned());

    Ok(())
}
