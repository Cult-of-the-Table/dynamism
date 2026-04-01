use anyhow::Result;
use dynamism::{db, segmentation, telemetry};
#[tokio::main]
async fn main() -> Result<()> {
    let (tel, tel_handle) = telemetry::spawn();
    let (_tx, rx, handle) = segmentation::worker::spawn(tel.clone());
    db::worker::spawn(rx, "database".to_owned(), "dynamism_main_table".to_owned());
    handle.await.unwrap();
    tel_handle.await.unwrap();

    Ok(())
}
