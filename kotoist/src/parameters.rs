use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex, RwLock,
};

use koto::Koto;
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
        let snippets = RwLock::new(vec![Snippet::default(); 128]);

        Self {
            koto: RwLock::new(koto),
            pipe_in,
            orchestrator: orchestrator.clone(),
            selected_snippet: Default::default(),
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

    pub(crate) fn set_snippet_name(&self, index: usize, name: &str) {
        if let Some(snippet) = self
            .snippets
            .write()
            .ok()
            .as_mut()
            .and_then(|s| s.get_mut(index))
        {
            snippet.name = name.to_owned();
        }
    }

    pub(crate) fn snippet_name_at(&self, index: usize) -> String {
        self.snippets.read().unwrap()[index].name.clone()
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
            Ok(v) => self.pipe_in.send(PipeMessage::Normal(
                koto.value_to_string(v).unwrap_or_default(),
            )),
            Err(err) => self
                .pipe_in
                .send(PipeMessage::Error(format!("Error\n\n{}", err))),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct Snippet {
    pub(crate) name: String,
    pub(crate) code: String,
    pub(crate) number: usize,
}
