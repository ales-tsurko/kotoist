use std::sync::{Arc, Mutex, RwLock};

use koto::Koto;
use nih_plug::prelude::*;
use serde::{Deserialize, Serialize};

use crate::interpreter;
use crate::orchestrator::Orchestrator;
use crate::pipe::PipeIn;

#[derive(Params)]
pub(crate) struct Parameters {
    koto: RwLock<Koto>,
    orchestrator: Arc<Mutex<Orchestrator>>,
    pipe_in: PipeIn,
    #[persist = "selected-pad"]
    selected_pad: RwLock<Pad>,
    #[persist = "snippets"]
    snippets: RwLock<Vec<String>>,
    #[persist = "pad-names"]
    pad_names: RwLock<Vec<String>>,
}

impl Parameters {
    pub(crate) fn new(pipe_in: PipeIn) -> Self {
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(pipe_in.clone())));
        let koto = interpreter::init_koto(orchestrator.clone(), pipe_in.clone());

        Self {
            koto: RwLock::new(koto),
            pipe_in,
            orchestrator: orchestrator.clone(),
            selected_pad: Default::default(),
            snippets: RwLock::new(Vec::with_capacity(128)),
            pad_names: RwLock::new(Vec::with_capacity(128)),
        }
    }

    pub(crate) fn set_selected_pad(&self, selection: Pad) {
        *self.selected_pad.write().unwrap() = selection;
    }

    pub(crate) fn selected_pad(&self) -> Pad {
        self.selected_pad.read().unwrap().clone()
    }

    pub(crate) fn set_pad_name(&self, pad: Pad) {
        let mut selected = self.selected_pad.write().unwrap();
        if selected.number == pad.number {
            selected.name = pad.name.clone();
        }
        self.pad_names.write().unwrap()[pad.number] = pad.name;
    }

    pub(crate) fn pad_name_at(&self, index: usize) -> String {
        self.pad_names.read().unwrap()[index].clone()
    }

    pub(crate) fn set_code(&self, code: &str) {
        self.snippets.write().unwrap()[self.selected_pad.read().unwrap().number] = code.to_string();
    }

    pub(crate) fn code(&self) -> String {
        self.snippets.read().unwrap()[self.selected_pad.read().unwrap().number].clone()
    }

    pub(crate) fn eval_snippet_at(&self, index: usize) {
        self.eval_code(&self.snippets.read().unwrap()[index]);
    }

    pub(crate) fn eval_code(&self, code: &str) {
        let mut koto = self.koto.write().unwrap();
        match koto.compile(code) {
            Ok(_) => {
                if let Err(err) = koto.run() {
                    self.pipe_in.send_err(&format!("Runtime error: {}\n", err));
                }
            }
            Err(err) => self.pipe_in.send_err(&format!("Compiler error: {}\n", err)),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct Pad {
    name: String,
    number: usize,
}
