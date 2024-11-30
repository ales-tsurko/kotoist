use std::sync::{Arc, Mutex};

use crate::orchestrator::{Orchestrator, Pattern, Scale};
use koto::prelude::*;

use crate::pipe::{Message as PipeMessage, PipeIn};

pub(crate) fn make_module(
    orchestrator: Arc<Mutex<Orchestrator>>,
    callbacks: Arc<Mutex<Callbacks>>,
    pipe_in: PipeIn,
) -> KMap {
    let result = KMap::new();

    let cbks = callbacks.clone();
    result.add_fn("on_load", move |ctx| cbks.lock().unwrap().set_load(ctx));
    let cbks = callbacks.clone();
    result.add_fn("on_midiin", move |ctx| cbks.lock().unwrap().set_midiin(ctx));
    let cbks = callbacks.clone();
    result.add_fn("on_midiincc", move |ctx| {
        cbks.lock().unwrap().set_midiincc(ctx)
    });
    let cbks = callbacks.clone();
    result.add_fn("on_pause", move |ctx| cbks.lock().unwrap().set_pause(ctx));
    result.add_fn("on_play", move |ctx| {
        callbacks.lock().unwrap().set_play(ctx)
    });
    result.add_fn("print_scales", move |ctx| {
        print_scales(ctx, pipe_in.clone())
    });
    result.add_fn("midiout", move |ctx| midiout(ctx, orchestrator.clone()));

    result
}

#[derive(Default)]
pub(crate) struct Callbacks {
    pub(crate) load: Option<KValue>,
    pub(crate) midiin: Option<KValue>,
    pub(crate) midiincc: Option<KValue>,
    pub(crate) pause: Option<KValue>,
    pub(crate) play: Option<KValue>,
}

impl Callbacks {
    fn set_load(&mut self, ctx: &mut CallContext) -> Result<KValue, koto::Error> {
        Self::set_callback(&mut self.load, ctx, "on_load")
    }

    fn set_midiin(&mut self, ctx: &mut CallContext) -> Result<KValue, koto::Error> {
        Self::set_callback(&mut self.midiin, ctx, "on_midiin")
    }

    fn set_midiincc(&mut self, ctx: &mut CallContext) -> Result<KValue, koto::Error> {
        Self::set_callback(&mut self.midiincc, ctx, "on_midiincc")
    }

    fn set_pause(&mut self, ctx: &mut CallContext) -> Result<KValue, koto::Error> {
        Self::set_callback(&mut self.pause, ctx, "on_pause")
    }

    fn set_play(&mut self, ctx: &mut CallContext) -> Result<KValue, koto::Error> {
        Self::set_callback(&mut self.play, ctx, "on_play")
    }

    fn set_callback(
        ptr: &mut Option<KValue>,
        ctx: &mut CallContext,
        name: &str,
    ) -> Result<KValue, koto::Error> {
        use KValue::*;
        match ctx.args() {
            [value] => {
                *ptr = Some(Self::check_callback(name, value.to_owned())?);
                Ok(Null)
            }
            _ => runtime_error!("kotoist.{}: expected a function", name),
        }
    }

    fn check_callback(func_name: &str, value: KValue) -> Result<KValue, koto::Error> {
        use KValue::*;
        if !matches!(value, Function(_) | CaptureFunction(_) | NativeFunction(..)) {
            runtime_error!("kotoist.{}: expected a function", func_name)
        } else {
            Ok(value)
        }
    }
}

fn print_scales(ctx: &mut CallContext, pipe_in: PipeIn) -> Result<KValue, koto::Error> {
    use KValue::Null;

    match ctx.args() {
        [] => {
            pipe_in.send(PipeMessage::Normal(Scale::list()));
            Ok(Null)
        }
        _ => runtime_error!("kotoist.print_scales: doesn't expect any arguments"),
    }
}

fn midiout(
    ctx: &mut CallContext,
    orchestrator: Arc<Mutex<Orchestrator>>,
) -> Result<KValue, koto::Error> {
    use KValue::{List, Map, Null, Number};
    match ctx.args() {
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
                            "kotoist.midiout: \
                                Expected arguments: map or list of maps, quantization."
                        )
                    }
                }
            }

            orchestrator.lock().unwrap().set_patterns(patterns, quant);

            Ok(Null)
        }
        _ => runtime_error!(
            "kotoist.midiout: \
                Expected arguments: map or list of maps, quantization."
        ),
    }
}
