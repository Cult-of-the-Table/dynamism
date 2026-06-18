use std::time::Duration;

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc::{Sender, channel};
use tokio::task::JoinHandle;
pub enum BarEvent {
    Inc(u64),
    SetPos(u64),
    SetLen(u64),
    AddLen(u64),
    SetMsg(String),
    Finish(String),
}
pub enum TelEvent {
    CreateSpinner {
        reply: tokio::sync::oneshot::Sender<tokio::sync::mpsc::Sender<BarEvent>>,
    },
    CreateBar {
        total: u64,
        style: ProgressStyle,
        reply: tokio::sync::oneshot::Sender<tokio::sync::mpsc::Sender<BarEvent>>,
    },
}
// pretty self explainatory, match on TelEvent,
// then match on BarEvent and update accordingly
pub fn spawn() -> (Sender<TelEvent>, JoinHandle<()>) {
    let multi = MultiProgress::new();
    let (tx, mut rx) = channel(10);
    let handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                TelEvent::CreateSpinner { reply } => {
                    let spinner = multi.insert(0, ProgressBar::new_spinner());
                    spinner.enable_steady_tick(Duration::from_millis(100));
                    spinner.set_message("test");
                    let (s_tx, mut s_rx) = tokio::sync::mpsc::channel(10);
                    let _ = reply.send(s_tx);
                    tokio::spawn(async move {
                        while let Some(event) = s_rx.recv().await {
                            match event {
                                BarEvent::Finish(s) => {
                                    spinner.finish_with_message(s);
                                    break;
                                }
                                _ => println!("bar event not allowed for spinner"),
                            }
                        }
                    });
                }
                TelEvent::CreateBar {
                    total,
                    style,
                    reply,
                } => {
                    let new_pb = multi.add(ProgressBar::new(total));
                    new_pb.set_style(style);
                    let (b_tx, mut b_rx) = tokio::sync::mpsc::channel(10);
                    let _ = reply.send(b_tx);
                    tokio::spawn(async move {
                        while let Some(event) = b_rx.recv().await {
                            match event {
                                BarEvent::Inc(s) => new_pb.inc(s),
                                BarEvent::SetPos(s) => new_pb.set_position(s),
                                BarEvent::SetLen(s) => new_pb.set_length(s),
                                BarEvent::AddLen(s) => new_pb.inc_length(s),
                                BarEvent::SetMsg(s) => new_pb.set_message(s),
                                BarEvent::Finish(s) => {
                                    new_pb.finish_with_message(s);
                                    break;
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    (tx, handle)
}
