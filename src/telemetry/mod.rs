use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc::{Sender, channel};
use tokio::task::JoinHandle;
pub enum TelEvent {
    NewData(u64),
    ProcessedData(u64),
}
pub fn spawn() -> (Sender<TelEvent>, JoinHandle<()>) {
    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::default_bar());
    let (tx, mut rx) = channel(10);
    let handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                TelEvent::NewData(count) => {
                    pb.inc_length(count);
                }
                TelEvent::ProcessedData(count) => {
                    pb.inc(count);
                }
            }
        }
        pb.finish_with_message("bar done?");
    });
    (tx, handle)
}
