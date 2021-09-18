use std::sync::Arc;

use koto::runtime::{runtime_error, Value, ValueMap};

#[derive(Debug, Default)]
pub(crate) struct Pattern;

pub(crate) fn make_module(pattern: Arc<Pattern>) -> ValueMap {
    use Value::{Iterator, Number, Str};

    let mut result = ValueMap::new();

    result.add_fn("midi_out", {
        |vm, args| match vm.get_args(args) {
            [Iterator(pattern), Number(channel), Number(quant)] => Ok(Str(format!(
                "pat {:?}, ch {}, q {}",
                pattern, channel, quant
            )
            .into())),
            _ => runtime_error!(
                "pattern.midi_out: Expected arguments: pattern, midi channel, quantization."
            ),
        }
    });

    result
}
