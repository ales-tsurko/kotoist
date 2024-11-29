use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use koto::prelude::*;
use koto_random::make_module as make_random_module;

use crate::orchestrator::{Orchestrator, Pattern, Scale};
use crate::pipe::{Message, PipeIn};

const KOTO_LIB_CODE: &str = include_str!("../koto/pattern.koto");

pub(crate) fn init_koto(orchestrator: Arc<Mutex<Orchestrator>>, pipe_in: PipeIn) -> Koto {
    let mut result = Koto::with_settings(
        KotoSettings {
            run_tests: cfg!(debug_assertions),
            export_top_level_ids: true,
            ..Default::default()
        }
        .with_stdin(StdIn)
        .with_stdout(StdOut::from(&pipe_in))
        .with_stderr(StdErr::from(&pipe_in)),
    );

    result
        .prelude()
        .insert("pattern", make_pattern_module(orchestrator, pipe_in));
    result.prelude().insert("random", make_random_module());

    result
        .compile("from pattern import midiout, print_scales")
        .expect("import statement should compile");
    result
        .run()
        .expect("importing pattern module should not fail");

    result
        .compile(KOTO_LIB_CODE)
        .expect("core pattern lib should compile");
    if let Err(e) = result.run() {
        eprintln!("{}", e);
        panic!("evaluating the koto pattern library should not fail");
    }

    result
}

fn make_pattern_module(orchestrator: Arc<Mutex<Orchestrator>>, pipe_in: PipeIn) -> KMap {
    use KValue::{List, Map, Null, Number};

    let result = KMap::new();

    result.add_fn("midiout", move |ctx| match ctx.args() {
        [Map(map), Number(quant)] => {
            let quant = f64::from(quant);

            match Pattern::try_from(map) {
                Ok(pattern) => {
                    orchestrator
                        .lock()
                        .unwrap()
                        .set_patterns(vec![pattern], quant);
                }
                Err(e) => return runtime_error!("{}", e),
            }

            Ok(Null)
        }
        [List(list), Number(quant)] => {
            let quant = f64::from(quant);
            let mut patterns = Vec::new();

            for item in list.clone().data().iter() {
                match item {
                    Map(map) => match Pattern::try_from(map) {
                        Ok(pattern) => {
                            patterns.push(pattern);
                        }
                        Err(e) => return runtime_error!("{}", e),
                    },
                    _ => {
                        return runtime_error!(
                            "pattern.midiout: \
                                Expected arguments: map or list of maps, quantization."
                        )
                    }
                }
            }

            orchestrator.lock().unwrap().set_patterns(patterns, quant);

            Ok(Null)
        }
        _ => runtime_error!(
            "pattern.midiout: \
                Expected arguments: map or list of maps, quantization."
        ),
    });

    result.add_fn("print_scales", move |ctx| match ctx.args() {
        [] => {
            pipe_in.send(Message::Normal(Scale::list()));
            Ok(Null)
        }
        _ => runtime_error!("pattern.print_scales: doesn't expect any arguments"),
    });

    result
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

impl_channel!(StdOut, Message::Normal, "koto-stdout");
impl_channel!(StdErr, Message::Error, "koto-stderr");
