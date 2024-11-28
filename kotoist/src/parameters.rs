use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc, Mutex, RwLock,
};
use std::thread;

use koto::prelude::*;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use serde::{Deserialize, Serialize};

use crate::editor::WINDOW_SIZE;
use crate::interpreter;
use crate::orchestrator::Orchestrator;
use crate::pipe::{Message as PipeMessage, PipeIn};

#[derive(Params)]
pub(crate) struct Parameters {
    interpreter_sender: mpsc::Sender<InterpreterMessage>,
    pub(crate) orchestrator: Arc<Mutex<Orchestrator>>,
    #[persist = "editor-state"]
    pub(crate) editor_state: Arc<EguiState>,
    #[persist = "selected-snippet"]
    pub(crate) selected_snippet: AtomicUsize,
    #[nested(array)]
    pub(crate) snippets: Arc<Vec<Snippet>>,
}

impl Parameters {
    pub(crate) fn new(pipe_in: PipeIn) -> Self {
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(pipe_in.clone())));
        let snippets: Arc<Vec<Snippet>> = Arc::new(
            (0..=127)
                .map(nn_to_pk)
                .map(Snippet::with_piano_key)
                .collect(),
        );
        let interpreter_sender =
            Self::spawn_interpreter_worker(orchestrator.clone(), snippets.clone(), pipe_in);

        Self {
            interpreter_sender,
            orchestrator: orchestrator.clone(),
            selected_snippet: Default::default(),
            snippets,
            editor_state: EguiState::from_size(WINDOW_SIZE.0, WINDOW_SIZE.1),
        }
    }

    fn spawn_interpreter_worker(
        orchestrator: Arc<Mutex<Orchestrator>>,
        snippets: Arc<Vec<Snippet>>,
        pipe_in: PipeIn,
    ) -> mpsc::Sender<InterpreterMessage> {
        let (interpreter_sender, interpreter_receiver) = mpsc::channel();

        fn eval_code(koto: &mut Koto, pipe_in: &PipeIn, code: &str) {
            match koto.compile_and_run(code) {
                Ok(v) => {
                    if !matches!(v, KValue::Null) {
                        pipe_in.send(PipeMessage::Normal(
                            koto.value_to_string(v).unwrap_or_default(),
                        ));
                    }
                }
                Err(err) => pipe_in.send(PipeMessage::Error(format!("Error: {}", err))),
            }
        }

        thread::spawn(move || {
            let mut koto = interpreter::init_koto(orchestrator, pipe_in.clone());
            loop {
                if let Ok(message) = interpreter_receiver.try_recv() {
                    match message {
                        InterpreterMessage::EvalSnippet(index) => {
                            let code = snippets[index].code.read().unwrap();
                            eval_code(&mut koto, &pipe_in, &code);
                        }

                        InterpreterMessage::SetSnippet(index, code) => {
                            *snippets[index].code.write().unwrap() = code;
                        }

                        InterpreterMessage::EvalCode(code) => eval_code(&mut koto, &pipe_in, &code),
                    }
                }
            }
        });

        interpreter_sender
    }

    pub(crate) fn set_selected_snippet_index(&self, index: usize) {
        self.selected_snippet.store(index, Ordering::Relaxed);
    }

    pub(crate) fn selected_snippet_index(&self) -> usize {
        self.selected_snippet.load(Ordering::SeqCst)
    }

    pub(crate) fn send_interpreter_msg(&self, msg: InterpreterMessage) {
        self.interpreter_sender
            .send(msg)
            .expect("sending to unbound channel should not fail");
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InterpreterMessage {
    EvalSnippet(usize),
    SetSnippet(usize, String),
    EvalCode(String),
}

// note number to piano key
const PITCH_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
fn nn_to_pk(nn: usize) -> PianoKey {
    let pitch = PITCH_NAMES[nn % PITCH_NAMES.len()];
    let octave = nn / 12;
    let name = format!("{pitch}{octave}");

    PianoKey {
        is_black: name.contains('#'),
        name,
    }
}

#[derive(Debug, Params, Default, Deserialize, Serialize)]
pub(crate) struct Snippet {
    #[persist = "code"]
    pub(crate) code: RwLock<String>,
    #[persist = "piano_key"]
    pub(crate) piano_key: RwLock<PianoKey>,
}

impl Snippet {
    fn with_piano_key(piano_key: PianoKey) -> Self {
        Self {
            piano_key: RwLock::new(piano_key),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct PianoKey {
    pub(crate) name: String,
    pub(crate) is_black: bool,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nn_to_pk() {
        assert_eq!(
            nn_to_pk(60),
            PianoKey {
                name: "C5".to_string(),
                is_black: false
            }
        )
    }
}
