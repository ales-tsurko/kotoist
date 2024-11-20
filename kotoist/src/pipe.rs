//! Pipes used to send output from the interpreter to the console.
use std::sync::mpsc::{self, Receiver, Sender};

pub(crate) fn new_pipe() -> (PipeIn, PipeOut) {
    let (sender, receiver) = mpsc::channel();
    (PipeIn { sender }, PipeOut { receiver })
}

/// The type for sending the output.
#[derive(Clone)]
pub(crate) struct PipeIn {
    pub(crate) sender: Sender<Message>,
}

impl PipeIn {
    pub(crate) fn send(&self, message: Message) {
        self.sender
            .send(message)
            .expect("sending to unbound channel should not fail");
    }
}

/// The type for reading the output (and showing it in console)
pub(crate) struct PipeOut {
    pub(crate) receiver: Receiver<Message>,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Normal(String),
    Error(String),
}
