use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc, Mutex, RwLock,
};
use std::thread;

use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use serde::{Deserialize, Serialize};

use crate::editor::{PianoRollEvent, WINDOW_SIZE};
use crate::interpreter::Interpreter;
use crate::orchestrator::Orchestrator;
use crate::pipe::PipeIn;

#[derive(Params)]
pub(crate) struct Parameters {
    interpreter_sender: mpsc::Sender<InterpreterMessage>,
    piano_roll_sender: mpsc::Sender<PianoRollEvent>,
    cursor_in_beats: Arc<AtomicF32>,
    gl_context_valid: Arc<AtomicBool>,
    pub(crate) orchestrator: Arc<Mutex<Orchestrator>>,
    #[persist = "editor-state"]
    pub(crate) editor_state: Arc<EguiState>,
    #[persist = "selected-snippet"]
    pub(crate) selected_snippet: AtomicUsize,
    #[persist = "snippets"]
    pub(crate) snippets: Arc<RwLock<Vec<Snippet>>>,
}

impl Parameters {
    pub(crate) fn new(pipe_in: PipeIn, piano_roll_sender: mpsc::Sender<PianoRollEvent>) -> Self {
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(pipe_in.clone())));
        // there always should be at least one snippet
        let snippets = Arc::new(RwLock::new(vec![Snippet::with_random_name()]));
        let interpreter_sender =
            Self::spawn_interpreter_worker(orchestrator.clone(), snippets.clone(), pipe_in);

        Self {
            interpreter_sender,
            cursor_in_beats: Default::default(),
            orchestrator: orchestrator.clone(),
            selected_snippet: Default::default(),
            piano_roll_sender,
            gl_context_valid: Default::default(),
            snippets,
            editor_state: EguiState::from_size(WINDOW_SIZE.0, WINDOW_SIZE.1),
        }
    }

    fn spawn_interpreter_worker(
        orchestrator: Arc<Mutex<Orchestrator>>,
        snippets: Arc<RwLock<Vec<Snippet>>>,
        pipe_in: PipeIn,
    ) -> mpsc::Sender<InterpreterMessage> {
        let (interpreter_sender, interpreter_receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut interp = Interpreter::new(orchestrator, pipe_in.clone());
            let mut is_playing = false;
            loop {
                if let Ok(message) = interpreter_receiver.recv() {
                    match message {
                        InterpreterMessage::SetSnippetCode(index, code) => {
                            if let Some(snippet) = snippets.write().unwrap().get_mut(index) {
                                snippet.code = code;
                            }
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

                        InterpreterMessage::AddSnippet => {
                            let mut snippets = snippets.write().unwrap();
                            snippets.push(Snippet::with_random_name());
                        }

                        InterpreterMessage::RemoveSnippet(id) => {
                            let mut snippets = snippets.write().unwrap();
                            if snippets.len() > 1 {
                                snippets.swap_remove(id);
                            }
                        }
                    }
                }
            }
        });

        interpreter_sender
    }

    pub(crate) fn clone_cursor(&self) -> Arc<AtomicF32> {
        self.cursor_in_beats.clone()
    }

    pub(crate) fn clone_gl_context_validity(&self) -> Arc<AtomicBool> {
        self.gl_context_valid.clone()
    }

    pub(crate) fn on_beats_position_changed(&self, value: f32) {
        self.cursor_in_beats.store(value, Ordering::Relaxed);
    }

    pub(crate) fn check_gl_context_valid(&self) {
        // when the editor has closed, gl context becomes invalid
        // we have to check frequently, to be able to catch every frame
        if !self.editor_state.is_open() {
            self.gl_context_valid.store(false, Ordering::Relaxed);
        }
    }

    pub(crate) fn send_piano_roll_note_on(&self, pitch: u8, channel: u8) {
        let pitch = pitch % 36;
        let _ = self
            .piano_roll_sender
            .send(PianoRollEvent::NoteOn { pitch, channel });
    }

    pub(crate) fn send_piano_roll_note_off(&self, pitch: u8, channel: u8) {
        let pitch = pitch % 36;
        let _ = self
            .piano_roll_sender
            .send(PianoRollEvent::NoteOff { pitch, channel });
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
    // EvalSnippet(Uuid),
    SetSnippetCode(usize, String),
    EvalCode(String),
    OnLoad,
    OnMidiIn(u8, f32, u8),
    OnMidiInCc(u8, f32, u8),
    OnPause,
    OnPlay,
    AddSnippet,
    RemoveSnippet(usize),
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Snippet {
    pub(crate) code: String,
    pub(crate) name: String,
}

const NAME_SYMBOLS: [&str; 28] = [
    "☃", "☄", "☠", "⚓", "🌊", "🌋", "🍄", "🍐", "🍭", "🎃", "🎩", "🐲", "👁", "👂", "👓", "👹",
    "👺", "👻", "👽", "👾", "👿", "💀", "🕷", "😀", "😇", "😈", "😱", "😶",
];

impl Snippet {
    pub(crate) fn with_random_name() -> Self {
        let name = fastrand::choose_multiple(NAME_SYMBOLS, 4)
            .into_iter()
            .collect::<String>();
        Self {
            name,
            code: String::new(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct PianoKey {
    pub(crate) name: String,
    pub(crate) is_black: bool,
}
