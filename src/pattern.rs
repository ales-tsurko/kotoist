use std::collections::hash_map::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use koto::runtime::{
    runtime_error, Value, ValueIterator, ValueIteratorOutput, ValueList, ValueMap, ValueNumber,
};
use thiserror::Error;
use vst::api::{EventType, MidiEvent, MidiEventFlags, TimeInfoFlags};
use vst::host::Host;
use vst::plugin::HostCallback;

use crate::parameters::Parameters;

pub(crate) fn make_module(scheduler: Arc<Mutex<Scheduler>>) -> ValueMap {
    use Value::{Empty, Iterator, Number};

    let mut result = ValueMap::new();

    result.add_fn("midi_out", {
        move |vm, args| match vm.get_args(args) {
            [Iterator(iterator), Number(quant)] => {
                let quant = match quant {
                    ValueNumber::F64(num) => *num,
                    ValueNumber::I64(num) => *num as f64,
                };
                let scheduler = scheduler.lock().unwrap();
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
    wait_until: f64,
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
        let mut position = self.position();
        let offset = quant - (position % quant);
        if self.is_playing() {
            position = offset + position;
        } else {
            position = offset;
        }
        *self.queued_pattern.write().unwrap() = Some(ScheduledPattern { position, pattern });
    }

    fn position(&self) -> f64 {
        self.host
            .get_time_info(TimeInfoFlags::TEMPO_VALID.bits())
            .unwrap()
            .sample_pos
    }

    fn is_playing(&self) -> bool {
        if let Some(time_info) = self.host.get_time_info(0) {
            return TimeInfoFlags::from_bits(time_info.flags)
                .map(|val| val.contains(TimeInfoFlags::TRANSPORT_PLAYING))
                .unwrap_or(false);
        }

        false
    }

    pub(crate) fn process(&mut self) -> Option<Vec<MidiEvent>> {
        if !self.is_playing() {
            return None;
        }

        let position = self.position();
        self.check_queued(position);

        let result = if let Some(pattern) = self.pattern.get_mut().unwrap() {
            if position < self.wait_until {
                return None;
            }
            pattern
                .pattern
                .next()
                .map(|event| self.process_events(position, event))
        } else {
            None
        };

        result.map(|event| {
            let beat_length = self.beat_length();
            let length = event.event.length * beat_length * event.event.dur;
            let length = length - (position % length) - 1.0;
            event.into_vst_midi(self.host.get_block_size() as f64, length)
        })
    }

    /// check if the queued pattern should play
    fn check_queued(&mut self, position: f64) {
        let queued = self.queued_pattern.get_mut().unwrap();
        if let Some(pattern) = queued.take() {
            if pattern.position <= position {
                *self.pattern.get_mut().unwrap() = Some(pattern);
            } else {
                *queued = Some(pattern);
            }
        }
    }

    fn process_events(&mut self, position: f64, event: Event) -> ScheduledEvent {
        let end = event.dur * self.beat_length();
        let offset = position % end;
        self.wait_until = position + end - offset;

        ScheduledEvent { position, event }
    }

    fn beat_length(&self) -> f64 {
        let time_info = self
            .host
            .get_time_info(TimeInfoFlags::TEMPO_VALID.bits())
            .unwrap();
        let beats_per_sec = time_info.tempo / 60.0;
        time_info.sample_rate / beats_per_sec
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

    fn next_in_iterator(&mut self, iterator: &mut ValueIterator) -> Option<Event> {
        if let Some(value) = iterator.next() {
            match value {
                ValueIteratorOutput::Value(val) => match val {
                    Value::Map(koto_event) => match self.event_from_koto(&koto_event) {
                        Ok(event) => return Some(event),
                        Err(e) => self.post_error(e),
                    },
                    Value::Iterator(iterator) => {
                        return self.next_in_iterator(&mut iterator.clone());
                    }
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
        for (key, value) in map.iter() {
            if *key == "note" {
                continue;
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

        let value =
            self.handle_notes(map["note"], event["velocity"] as u8, event["channel"] as u8)?;

        Ok(Event {
            value,
            dur: event["dur"],
            length: event["length"],
        })
    }

    fn handle_notes(
        &self,
        value: &Value,
        velocity: u8,
        channel: u8,
    ) -> Result<Vec<EventValue>, PatternError> {
        if self.is_rest(value)? {
            return Ok(vec![EventValue::Rest]);
        }

        let value = match value {
            Value::Number(num) => match num {
                ValueNumber::F64(val) => vec![*val],
                ValueNumber::I64(val) => vec![*val as f64],
            },
            Value::List(list) => self.handle_notes_list(list)?,
            _ => return Err(PatternError::TypeError),
        };

        Ok(value
            .iter()
            .map(|nn| EventValue::Note(*nn as u8, velocity, channel))
            .collect())
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

    fn handle_notes_list(&self, list: &ValueList) -> Result<Vec<f64>, PatternError> {
        list.data()
            .iter()
            .map(|val| match val {
                Value::Number(num) => match num {
                    ValueNumber::F64(v) => Ok(*v),
                    ValueNumber::I64(v) => Ok(*v as f64),
                },
                _ => Err(PatternError::TypeError),
            })
            .collect()
    }

    fn post_error(&self, error: PatternError) {
        self.parameters.append_console(&format!("{}\n", error));
    }
}

impl Iterator for Pattern {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iterator = self.iterator.clone();
        self.next_in_iterator(&mut iterator)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScheduledEvent {
    position: f64,
    event: Event,
}

impl ScheduledEvent {
    pub(crate) fn into_vst_midi(self, block_size: f64, length: f64) -> Vec<MidiEvent> {
        let delta_frames = block_size - (self.position % block_size);

        self.event
            .value
            .iter()
            .filter_map(|value| match value {
                EventValue::Note(nn, vel, ch) => Some([0x90 + ch, *nn, *vel]),
                &EventValue::Rest => None,
            })
            .map(|midi_data| MidiEvent {
                event_type: EventType::Midi,
                byte_size: 8,
                delta_frames: delta_frames as i32,
                flags: MidiEventFlags::REALTIME_EVENT.bits(),
                note_length: length as i32,
                note_offset: 0,
                midi_data,
                _midi_reserved: 0,
                detune: 0,
                note_off_velocity: 0,
                _reserved1: 0,
                _reserved2: 0,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Event {
    pub(crate) value: Vec<EventValue>,
    pub(crate) dur: f64,
    pub(crate) length: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum EventValue {
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
