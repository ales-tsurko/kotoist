use std::sync::{Arc, Mutex, MutexGuard};

use koto::prelude::*;
use koto_random::make_module as make_random_module;

use super::kotoist_module::{self, Callbacks};
use crate::orchestrator::Orchestrator;
use crate::pipe::{Message as PipeMessage, PipeIn};

const KOTO_LIB_CODE: &str = include_str!("../../koto/pattern.koto");

pub(crate) struct Interpreter {
    koto: Koto,
    pipe_in: PipeIn,
    callbacks: Arc<Mutex<Callbacks>>,
}

impl Interpreter {
    pub(crate) fn new(orchestrator: Arc<Mutex<Orchestrator>>, pipe_in: PipeIn) -> Self {
        let mut koto = Koto::with_settings(
            KotoSettings {
                run_tests: cfg!(debug_assertions),
                export_top_level_ids: true,
                ..Default::default()
            }
            .with_stdin(StdIn)
            .with_stdout(StdOut::from(&pipe_in))
            .with_stderr(StdErr::from(&pipe_in)),
        );
        let callbacks = Arc::new(Mutex::new(Callbacks::default()));

        koto.prelude().insert(
            "kotoist",
            kotoist_module::make_module(orchestrator, callbacks.clone(), pipe_in.clone()),
        );
        koto.prelude().insert("random", make_random_module());

        koto.compile("from kotoist import midiout, on_load, on_midiin, \
            on_midiincc, on_play, on_pause, print_scales")
            .expect("import statement should compile");
        koto.run()
            .expect("importing pattern module should not fail");

        koto.compile(KOTO_LIB_CODE)
            .expect("core pattern lib should compile");
        if let Err(e) = koto.run() {
            eprintln!("{}", e);
            panic!("evaluating the koto pattern library should not fail");
        }

        Self {
            koto,
            callbacks,
            pipe_in,
        }
    }

    pub(crate) fn eval_code(&mut self, code: &str) {
        let result = self.koto.compile_and_run(code);
        self.handle_koto_result(result);
    }

    /// Dispatch `on_load` callback.
    pub(crate) fn on_load(&mut self) {
        self.dispatch_callback(&[], |cbs| cbs.load.clone())
    }

    /// Dispatch `on_midiin` callback.
    pub(crate) fn on_midiin(&mut self, nn: u8, vel: f32, ch: u8) {
        self.dispatch_callback(&[nn.into(), vel.into(), ch.into()], |cbs| {
            cbs.midiin.clone()
        })
    }

    /// Dispatch `on_midiincc` callback.
    pub(crate) fn on_midiincc(&mut self, cc: u8, vel: f32, ch: u8) {
        self.dispatch_callback(&[cc.into(), vel.into(), ch.into()], |cbs| {
            cbs.midiin.clone()
        })
    }

    /// Dispatch `on_pause` callback.
    pub(crate) fn on_pause(&mut self) {
        self.dispatch_callback(&[], |cbs| cbs.pause.clone())
    }

    /// Dispatch `on_play` callback.
    pub(crate) fn on_play(&mut self) {
        self.dispatch_callback(&[], |cbs| cbs.play.clone())
    }

    fn dispatch_callback<'a>(
        &mut self,
        args: impl Into<CallArgs<'a>>,
        map: impl FnOnce(MutexGuard<Callbacks>) -> Option<KValue>,
    ) {
        if let Some(cb) = self.callbacks.try_lock().ok().and_then(map) {
            let result = self.koto.call_function(cb, args);
            self.handle_koto_result(result);
        }
    }

    fn handle_koto_result(&mut self, result: Result<KValue, koto::Error>) {
        match result {
            Ok(v) => {
                if !matches!(v, KValue::Null) {
                    self.pipe_in.send(PipeMessage::Normal(
                        self.koto.value_to_string(v).unwrap_or_default(),
                    ));
                }
            }
            Err(err) => self
                .pipe_in
                .send(PipeMessage::Error(format!("Error: {}", err))),
        }
    }
}

struct StdIn;

impl KotoRead for StdIn {}
impl KotoWrite for StdIn {}

impl KotoFile for StdIn {
    fn id(&self) -> KString {
        "koto-stdin".into()
    }
}

trait PipeChannel {
    fn send(&self, value: String);
}

macro_rules! impl_channel {
    ($name:ident, $msg_constructor:expr, $id:expr) => {
        struct $name(PipeIn);

        impl PipeChannel for $name {
            fn send(&self, value: String) {
                self.0.send($msg_constructor(value));
            }
        }

        impl From<&PipeIn> for $name {
            fn from(value: &PipeIn) -> Self {
                Self(value.clone())
            }
        }

        impl KotoRead for $name {}

        impl KotoWrite for $name {
            fn write(&self, bytes: &[u8]) -> koto::Result<()> {
                let value = String::from_utf8_lossy(bytes);
                self.send(value.to_string());
                Ok(())
            }
            fn write_line(&self, text: &str) -> koto::Result<()> {
                self.send(text.to_owned());
                Ok(())
            }
        }

        impl KotoFile for $name {
            fn id(&self) -> KString {
                $id.into()
            }
        }
    };
}

impl_channel!(StdOut, PipeMessage::Normal, "koto-stdout");
impl_channel!(StdErr, PipeMessage::Error, "koto-stderr");
