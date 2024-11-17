use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use koto::prelude::*;
use koto_random::make_module as make_random_module;

use crate::orchestrator::{Orchestrator, Pattern, Scale};
use crate::pipe::PipeIn;

const KOTO_LIB_CODE: &str = include_str!("../koto/pattern.koto");

pub(crate) fn init_koto(orchestrator: Arc<Mutex<Orchestrator>>, pipe_in: PipeIn) -> Koto {
    let mut result = Koto::with_settings(KotoSettings {
        run_tests: cfg!(debug_assertions),
        ..Default::default()
    });

    result
        .prelude()
        .insert("pattern", make_pattern_module(orchestrator, pipe_in));
    result.prelude().insert("random", make_random_module());

    result
        .compile("from pattern import midi_out, print_scales")
        .expect("import statement should compile");
    result
        .run()
        .expect("importing pattern module should not fail");

    result
        .compile(KOTO_LIB_CODE)
        .expect("core pattern lib should compile");
    result
        .run()
        .expect("evaluating the koto pattern library should not fail");

    result
}

fn make_pattern_module(orchestrator: Arc<Mutex<Orchestrator>>, pipe_in: PipeIn) -> KMap {
    use KValue::{List, Map, Null, Number};

    let result = KMap::new();

    result.add_fn("midi_out", move |ctx| match ctx.args() {
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
                            "pattern.midi_out: \
                                Expected arguments: map or list of maps, quantization."
                        )
                    }
                }
            }

            orchestrator.lock().unwrap().set_patterns(patterns, quant);

            Ok(Null)
        }
        _ => runtime_error!(
            "pattern.midi_out: \
                Expected arguments: map or list of maps, quantization."
        ),
    });

    result.add_fn("print_scales", move |ctx| match ctx.args() {
        [] => {
            pipe_in.send_out(&Scale::list());
            Ok(Null)
        }
        _ => runtime_error!("pattern.print_scales: doesn't expect any arguments"),
    });

    result
}
