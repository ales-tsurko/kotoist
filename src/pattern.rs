use std::sync::Arc;

use koto::runtime::{
    runtime_error, Value, ValueIterator, ValueIteratorOutput, ValueList, ValueMap, ValueNumber,
};
use vst::plugin::HostCallback;

use crate::parameters::Parameters;

pub(crate) fn make_module(scheduler: Arc<Scheduler>) -> ValueMap {
    use Value::{Empty, Iterator, Number};

    let mut result = ValueMap::new();

    result.add_fn("midi_out", {
        move |vm, args| match vm.get_args(args) {
            [Iterator(iterator), Number(quant)] => {
                let quant = match quant {
                    ValueNumber::F64(num) => *num,
                    ValueNumber::I64(num) => *num as f64,
                };
                scheduler.schedule_pattern(
                    Pattern::new(iterator.to_owned(), Arc::clone(&scheduler.parameters)),
                    quant,
                );
                Ok(Empty)
            }
            _ => runtime_error!("pattern.midi_out: Expected arguments: pattern, quantization."),
        }
    });

    result
}

#[derive(Default)]
pub(crate) struct Scheduler {
    host: HostCallback,
    parameters: Arc<Parameters>,
    queue: Vec<Pattern>,
}

impl Scheduler {
    pub(crate) fn new(host: HostCallback, parameters: Arc<Parameters>) -> Self {
        Self {
            host,
            parameters,
            queue: Vec::new(),
        }
    }

    fn schedule_pattern(&self, pattern: Pattern, quant: f64) {}

    fn process() -> Option<Vec<Event>> {
        todo!()
    }
}

pub(crate) struct Pattern {
    iterator: ValueIterator,
    parameters: Arc<Parameters>,
}

impl Pattern {
    fn new(iterator: ValueIterator, parameters: Arc<Parameters>) -> Self {
        Self {
            iterator,
            parameters,
        }
    }

    fn parse_koto_list(&self, list: ValueList) -> Vec<Event> {
        list.data().iter().map(Event::from).collect()
    }
}

impl Iterator for Pattern {
    type Item = Vec<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.iterator.next() {
            match value {
                Ok(ValueIteratorOutput::Value(val)) => match val {
                    Value::List(list) => return Some(self.parse_koto_list(list)),
                    Value::Empty => return None,
                    _ => self
                        .parameters
                        .append_console("Unexpected return type value from pattern"),
                },
                Ok(ValueIteratorOutput::ValuePair(_, _)) => self
                    .parameters
                    .append_console("Unexpected return type value from pattern"),
                Err(err) => self.parameters.append_console(&format!("{}", err)),
            }
        }

        None
    }
}

#[derive(Debug)]
pub(crate) struct Event {
    pub(crate) e_type: EventType,
    pub(crate) state: EventState,
    pub(crate) dur: f64,
    pub(crate) length: f64,
}

impl From<&Value> for Event {
    fn from(value: &Value) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum EventType {
    Note(u8, u8, u8), // note number, velocity, channel number
    Pause,
}

#[derive(Debug)]
pub(crate) enum EventState {
    On,
    Off,
}
