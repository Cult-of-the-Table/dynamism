use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc::{Sender, channel};
use tokio::task::JoinHandle;
pub enum TelEvent {
    NewData(u64),
    ProcessedData(u64),
    Spinner(ProgressBar),
}
pub fn spawn() -> (Sender<TelEvent>, JoinHandle<()>) {
    let multi = MultiProgress::new();
    let mut pb: Option<ProgressBar> = None;
    let (tx, mut rx) = channel(10);
    let handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                TelEvent::NewData(count) => {
                    let pb = pb.get_or_insert_with(|| {
                        let pb = multi.add(ProgressBar::new(0));
                        pb.set_style(ProgressStyle::default_bar());
                        pb
                    });
                    pb.inc_length(count);
                }
                TelEvent::ProcessedData(count) => {
                    let pb = pb.get_or_insert_with(|| {
                        let pb = multi.add(ProgressBar::new(0));
                        pb.set_style(ProgressStyle::default_bar());
                        pb
                    });
                    pb.inc(count)
                }
                TelEvent::Spinner(pb) => {
                    multi.add(pb);
                }
            }
        }
        if let Some(pb) = pb {
            pb.finish_with_message("bar done?");
        }
    });
    (tx, handle)
}
