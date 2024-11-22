use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex, RwLock,
};

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
    koto: RwLock<Koto>,
    orchestrator: Arc<Mutex<Orchestrator>>,
    pipe_in: PipeIn,
    #[persist = "editor-state"]
    pub(crate) editor_state: Arc<EguiState>,
    #[persist = "selected-snippet"]
    pub(crate) selected_snippet: AtomicUsize,
    #[persist = "snippets"]
    pub(crate) snippets: RwLock<Vec<Snippet>>,
}

impl Parameters {
    pub(crate) fn new(pipe_in: PipeIn) -> Self {
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(pipe_in.clone())));
        let koto = interpreter::init_koto(orchestrator.clone(), pipe_in.clone());
        let snippets = RwLock::new(
            (0..=127)
                .map(nn_to_pk)
                .map(Snippet::with_piano_key)
                .collect(),
        );

        Self {
            koto: RwLock::new(koto),
            pipe_in,
            orchestrator: orchestrator.clone(),
            selected_snippet: 60.into(),
            snippets,
            editor_state: EguiState::from_size(WINDOW_SIZE.0, WINDOW_SIZE.1),
        }
    }

    pub(crate) fn set_selected_snippet_index(&self, index: usize) {
        self.selected_snippet.store(index, Ordering::Relaxed);
    }

    pub(crate) fn selected_snippet_index(&self) -> usize {
        self.selected_snippet.load(Ordering::SeqCst)
    }

    pub(crate) fn set_code(&self, index: usize, code: &str) {
        if let Some(snippet) = self
            .snippets
            .write()
            .ok()
            .as_mut()
            .and_then(|s| s.get_mut(index))
        {
            snippet.code = code.to_owned();
        }
    }

    pub(crate) fn code(&self, index: usize) -> String {
        self.snippets.read().unwrap()[index].code.clone()
    }

    pub(crate) fn eval_snippet_at(&self, index: usize) {
        self.eval_code(&self.snippets.read().unwrap()[index].code)
    }

    pub(crate) fn eval_code(&self, code: &str) {
        let mut koto = self.koto.write().unwrap();
        match koto.compile_and_run(code) {
            Ok(v) => {
                if !matches!(v, KValue::Null) {
                    self.pipe_in.send(PipeMessage::Normal(
                        koto.value_to_string(v).unwrap_or_default(),
                    ));
                }
            }
            Err(err) => self
                .pipe_in
                .send(PipeMessage::Error(format!("Error: {}", err))),
        }
    }
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct Snippet {
    pub(crate) code: String,
    pub(crate) piano_key: PianoKey,
}

impl Snippet {
    fn with_piano_key(piano_key: PianoKey) -> Self {
        Self {
            piano_key,
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
