use std::collections::hash_map::HashMap;
use std::sync::{Arc, RwLock};

use koto::runtime::{
    runtime_error, Value, ValueIterator, ValueIteratorOutput, ValueList, ValueMap, ValueNumber,
};
use thiserror::Error;
use vst::api::TimeInfoFlags;
use vst::host::Host;
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
    queued_pattern: RwLock<Option<ScheduledPattern>>,
    pattern: RwLock<Option<ScheduledPattern>>,
}

impl Scheduler {
    pub(crate) fn new(host: HostCallback, parameters: Arc<Parameters>) -> Self {
        Self {
            host,
            parameters,
            ..Default::default()
        }
    }

    fn schedule_pattern(&self, pattern: Pattern, quant: f64) {
        let position = self.current_beat_position();
        let offset = quant - (position % quant);
        let position = offset + position;
        *self.queued_pattern.write().unwrap() = Some(ScheduledPattern { position, pattern });
    }

    fn current_beat_position(&self) -> f64 {
        let time_info = self
            .host
            .get_time_info(TimeInfoFlags::TEMPO_VALID.bits())
            .unwrap();
        let beats_per_sec = time_info.tempo / 60.0;
        let beat_length = time_info.sample_rate / beats_per_sec;
        time_info.sample_pos / beat_length
    }

    // TODO return ScheduledEvent instead (i.e. contains start_time and end_time instead of dur +
    // length)?
    // We push note on and note off into different streams. Then we check next values in those
    // streams independently.
    fn process(&mut self) -> Option<Vec<Event>> {
        let position = self.current_beat_position();
        if let Some(pattern) = self.queued_pattern.get_mut().unwrap().take() {
            if pattern.position >= position {
                *self.pattern.write().unwrap() = Some(pattern);
            }
        }

        if let Some(pattern) = self.pattern.get_mut().unwrap() {
            let events = pattern.pattern.next();
            // TODO schedule events
            todo!();
        }
        
        None
    }
}

struct ScheduledPattern {
    position: f64,
    pattern: Pattern,
}

struct Pattern {
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

    fn next_in_iterator(&mut self, iterator: &mut ValueIterator) -> Option<Vec<Event>> {
        if let Some(value) = iterator.next() {
            match value {
                ValueIteratorOutput::Value(val) => match val {
                    Value::List(list) => return self.handle_koto_list(list),
                    Value::Empty => return None,
                    _ => self.post_error(PatternError::TypeError),
                },
                ValueIteratorOutput::ValuePair(_, _) => self.post_error(PatternError::TypeError),
                ValueIteratorOutput::Error(err) => {
                    self.parameters.append_console(&format!("{}", err))
                }
            }
        }

        None
    }

    fn handle_koto_list(&mut self, list: ValueList) -> Option<Vec<Event>> {
        match self.collect_events(list) {
            Ok(events) => {
                if events.is_empty() {
                    None
                } else {
                    Some(events)
                }
            }
            Err(err) => {
                self.post_error(err);
                None
            }
        }
    }

    fn collect_events(&mut self, list: ValueList) -> Result<Vec<Event>, PatternError> {
        let mut result = Vec::new();
        for value in list.data().iter() {
            result.append(&mut self.events_from_koto(value)?);
        }
        Ok(result)
    }

    fn events_from_koto(&mut self, value: &Value) -> Result<Vec<Event>, PatternError> {
        match value {
            Value::Map(koto_event) => Ok(vec![self.event_from_koto(koto_event)?]),
            Value::Iterator(iterator) => Ok(self
                .next_in_iterator(&mut iterator.clone())
                .unwrap_or_default()),
            _ => Err(PatternError::TypeError),
        }
    }

    fn event_from_koto(&mut self, koto_event: &ValueMap) -> Result<Event, PatternError> {
        let map = koto_event.data();
        let keys = vec!["dur", "length", "channel", "note", "velocity"];
        let values: Result<Vec<&Value>, PatternError> = keys
            .iter()
            .map(|k| map.get_with_string(k).ok_or(PatternError::EventTypeError))
            .collect();
        let mut event = HashMap::new();
        values?.into_iter().enumerate().for_each(|(n, val)| {
            event.insert(keys[n], val);
        });

        self.event_from_map(event)
    }

    fn event_from_map(&self, map: HashMap<&str, &Value>) -> Result<Event, PatternError> {
        let mut event = HashMap::new();
        let mut is_rest = false;
        for (key, value) in map.iter() {
            if *key == "note" {
                if self.is_rest(value)? {
                    is_rest = true;
                    continue;
                }
            }

            let value = match value {
                Value::Number(num) => match num {
                    ValueNumber::F64(val) => *val,
                    ValueNumber::I64(val) => *val as f64,
                },
                _ => return Err(PatternError::TypeError),
            };
            event.insert(*key, value);
        }

        let e_type = if is_rest {
            EventType::Rest
        } else {
            EventType::Note(
                event["note"] as u8,
                event["velocity"] as u8,
                event["channel"] as u8,
            )
        };

        Ok(Event {
            e_type,
            dur: event["dur"],
            length: event["length"],
        })
    }

    fn is_rest(&self, value: &Value) -> Result<bool, PatternError> {
        if let Value::Str(val) = value {
            if val.as_str() == "rest" {
                return Ok(true);
            }

            return Err(PatternError::TypeError);
        }

        Ok(false)
    }

    fn post_error(&self, error: PatternError) {
        self.parameters.append_console(&format!("{}", error));
    }
}

impl Iterator for Pattern {
    type Item = Vec<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iterator = self.iterator.clone();
        self.next_in_iterator(&mut iterator)
    }
}

#[derive(Debug)]
pub(crate) struct Event {
    pub(crate) e_type: EventType,
    pub(crate) dur: f64,
    pub(crate) length: f64,
}

#[derive(Debug)]
pub(crate) enum EventType {
    Note(u8, u8, u8), // note number, velocity, channel number
    Rest,
}

#[derive(Debug, Error)]
enum PatternError {
    #[error("Unexpected return value type from pattern")]
    TypeError,
    #[error("Event type is wrong or incomplete")]
    EventTypeError,
}
