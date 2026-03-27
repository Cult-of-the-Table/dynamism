use std::sync::mpsc::{channel, Receiver, Sender};

pub mod model;

pub fn spawn() -> (Sender<model::EmbeddingTask>, Receiver<model::EmbeddingTask>) {
    let (tx, _rx) = channel();
    let (_tx2, rx2) = channel();

    std::thread::spawn(move || loop {
        // acquire work
        match _rx.recv() {
            Ok(EmbeddingTask { text, range }) => {
                &text[range];
            }
        }
    });

    (tx, rx2)
}
