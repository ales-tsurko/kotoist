use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc, Mutex, RwLock,
};
use std::thread;

use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use serde::{Deserialize, Serialize};

use crate::editor::WINDOW_SIZE;
use crate::interpreter::Interpreter;
use crate::orchestrator::Orchestrator;
use crate::pipe::PipeIn;

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

        thread::spawn(move || {
            let mut interp = Interpreter::new(orchestrator, pipe_in.clone());
            let mut is_playing = false;
            loop {
                if let Ok(message) = interpreter_receiver.recv() {
                    match message {
                        InterpreterMessage::EvalSnippet(index) => {
                            let code = snippets[index].code.read().unwrap();
                            interp.eval_code(&code);
                        }

                        InterpreterMessage::SetSnippet(index, code) => {
                            *snippets[index].code.write().unwrap() = code;
                        }

                        InterpreterMessage::EvalCode(code) => interp.eval_code(&code),

                        InterpreterMessage::OnLoad => interp.on_load(),

                        InterpreterMessage::OnMidiIn(nn, vel, ch) => interp.on_midiin(nn, vel, ch),

                        InterpreterMessage::OnMidiInCc(cc, vel, ch) => {
                            interp.on_midiincc(cc, vel, ch)
                        }

                        InterpreterMessage::OnPause => {
                            if is_playing {
                                is_playing = false;
                                interp.on_pause();
                            }
                        }

                        InterpreterMessage::OnPlay => {
                            if !is_playing {
                                is_playing = true;
                                interp.on_play();
                            }
                        }
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
    OnLoad,
    OnMidiIn(u8, f32, u8),
    OnMidiInCc(u8, f32, u8),
    OnPause,
    OnPlay,
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
