//! Pipes used to send output from the interpreter to the console.
use std::sync::mpsc::{self, Receiver, Sender};

pub(crate) fn new_pipe() -> (PipeIn, PipeOut) {
    let (stdout, stdout_recv) = mpsc::channel();
    let (stderr, stderr_recv) = mpsc::channel();
    (
        PipeIn { stdout, stderr },
        PipeOut {
            stdout: stdout_recv,
            stderr: stderr_recv,
        },
    )
}

/// The type for sending the output.
#[derive(Clone)]
pub(crate) struct PipeIn {
    pub(crate) stderr: Sender<String>,
    pub(crate) stdout: Sender<String>,
}

impl PipeIn {
    pub(crate) fn send_out(&self, out: &str) {
        self.stdout
            .send(out.to_owned())
            .expect("sending to unbound stdout channel should not fail");
    }

    pub(crate) fn send_err(&self, err: &str) {
        self.stderr
            .send(err.to_owned())
            .expect("sending to unbound stderr channel should not fail");
    }
}

/// The type for reading the output (and showing it in console)
pub(crate) struct PipeOut {
    pub(crate) stderr: Receiver<String>,
    pub(crate) stdout: Receiver<String>,
}
