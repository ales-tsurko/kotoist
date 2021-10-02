mod scale;

use std::collections::hash_map::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex, RwLock};

use koto::runtime::{
    runtime_error, DataMap, Value, ValueIterator, ValueIteratorOutput, ValueList, ValueMap,
    ValueNumber,
};
use thiserror::Error;
use vst::api::{EventType, MidiEvent, MidiEventFlags, TimeInfoFlags};
use vst::host::Host;
use vst::plugin::HostCallback;

use crate::parameters::Parameters;
use scale::Scale;

pub(crate) fn make_module(player: Arc<Mutex<Player>>) -> ValueMap {
    use Value::{Empty, Iterator, List, Map, Number};

    let mut result = ValueMap::new();

    let player_cl = player.clone();

    result.add_fn("midi_out", {
        move |vm, args| match vm.get_args(args) {
            [Map(map), Number(quant)] => {
                let quant = f64::from(quant);
                match EventPattern::try_from(map) {
                    Ok(pattern) => todo!(),
                    Err(e) => runtime_error!("{}", e),
                }
            }
            [List(list), Number(quant)] => Ok(Empty),
            [Iterator(iterator), Number(quant)] => {
                let quant = f64::from(quant);
                let player = player_cl.lock().unwrap();
                player.schedule_stream(
                    EventStream::new(iterator.to_owned(), Arc::clone(&player.parameters)),
                    quant,
                );
                Ok(Empty)
            }
            _ => runtime_error!(
                "pattern.midi_out: \
                Expected arguments: iterator or map or list of them, quantization."
            ),
        }
    });

    let player_cl = player.clone();

    result.add_fn("print_scales", {
        move |vm, args| match vm.get_args(args) {
            [] => {
                player_cl
                    .lock()
                    .unwrap()
                    .parameters
                    .append_console(&Scale::list());
                Ok(Empty)
            }
            _ => runtime_error!("pattern.print_scales: doesn't expect any arguments"),
        }
    });

    result
}

struct EventPattern {
    dur: StreamF64,
    length: StreamF64,
    degree: StreamVecDegree,
    scale: StreamScale,
    root: StreamF64,
    transpose: StreamF64,
    mtranspose: StreamF64,
    octave: StreamF64,
    channel: StreamF64,
    amp: StreamF64,
}

impl EventPattern {
    fn try_next(&mut self) -> Result<Option<Event>, StreamError> {
        macro_rules! extract_value {
            ($name:ident) => {
                match self.$name.try_next()? {
                    Some(value) => value,
                    None => return Ok(None),
                }
            };
        }

        let dur = extract_value!(dur);
        let length = extract_value!(length);
        let degree = extract_value!(degree);
        let scale = extract_value!(scale);
        let root = extract_value!(root);
        let transpose = extract_value!(transpose);
        let mtranspose = extract_value!(mtranspose);
        let octave = extract_value!(octave);
        let channel = extract_value!(channel) as u8;
        let amp = extract_value!(amp);

        let velocity = (127.0 * amp).max(0.0).min(127.0) as u8;

        let pitches = self.make_pitches(degree, root, octave, scale, transpose, mtranspose);

        let value: Vec<EventValue> = pitches
            .iter()
            .map(|pitch| match pitch {
                Degree::Pitch(pitch) => EventValue::Note(*pitch as u8, velocity, channel),
                Degree::Rest => EventValue::Rest,
            })
            .collect();

        Ok(Some(Event { value, dur, length }))
    }

    fn make_pitches(
        &self,
        degree: Vec<Degree>,
        root: f64,
        octave: f64,
        scale: Scale,
        transpose: f64,
        mtranspose: f64,
    ) -> Vec<Degree> {
        let pitch_set: &[f64] = scale.into();
        let octave = (12.0 * octave).max(0.0).min(120.0);
        let root = root + octave + transpose;
        // this way we handle the case when mtranspose is negative as well
        let mtranspose = (pitch_set.len() + mtranspose as usize).max(0) % pitch_set.len();
        let root = root + pitch_set[mtranspose];

        degree
            .iter()
            .map(|d| match d {
                Degree::Pitch(p) => {
                    let pitch = (pitch_set.len() + *p as usize).max(0) % pitch_set.len();
                    Degree::Pitch(pitch_set[pitch] + root)
                }
                _ => Degree::Rest,
            })
            .collect()
    }
}

impl TryFrom<&ValueMap> for EventPattern {
    type Error = StreamError;

    fn try_from(map: &ValueMap) -> Result<Self, Self::Error> {
        let map = map.data();

        let dur = StreamF64::from_map(&map, "dur", 1.0)?;
        let length = StreamF64::from_map(&map, "length", 1.0)?;
        let degree = StreamVecDegree::from_map(&map, "degree", 0.0)?;
        let scale = StreamScale::from_map(&map, "scale", "chromatic")?;
        let root = StreamF64::from_map(&map, "root", 0.0)?;
        let transpose = StreamF64::from_map(&map, "transpose", 0.0)?;
        let mtranspose = StreamF64::from_map(&map, "mtranspose", 0.0)?;
        let octave = StreamF64::from_map(&map, "octave", 5.0)?;
        let channel = StreamF64::from_map(&map, "channel", 0.0)?;
        let amp = StreamF64::from_map(&map, "amp", 0.85)?;

        Ok(Self {
            dur,
            length,
            degree,
            scale,
            root,
            transpose,
            mtranspose,
            octave,
            channel,
            amp,
        })
    }
}

trait Stream {
    type Item;
    type Error;

    fn from_koto_value(value: &Value) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error>;
}

struct StreamF64 {
    value: Option<f64>,
    iterator: Option<ValueIterator>,
}

impl StreamF64 {
    fn from_map(map: &DataMap, key: &str, default: f64) -> Result<Self, StreamError> {
        match map.get_with_string(key) {
            Some(value) => StreamF64::from_koto_value(value),
            None => StreamF64::from_koto_value(&Value::Number(default.into())),
        }
    }
}

impl Stream for StreamF64 {
    type Item = f64;
    type Error = StreamError;

    fn from_koto_value(value: &Value) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match value {
            Value::Number(num) => Ok(Self {
                value: Some(f64::from(num)),
                iterator: None,
            }),
            Value::Iterator(iterator) => Ok(Self {
                value: None,
                iterator: Some(iterator.clone()),
            }),
            value => Err(StreamError::ValueTypeError(
                format!("{}", value),
                "number or iterator".to_string(),
            )),
        }
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(value) = self.value {
            Ok(Some(value))
        } else {
            // we expect iterator here, if there's no value
            self.iterator
                .as_mut()
                .expect("the iterator is unexpectedly None")
                .next()
                .map(|val| match val {
                    ValueIteratorOutput::Value(value) => match value {
                        Value::Number(num) => Ok(f64::from(num)),
                        other => Err(StreamError::ReturnTypeError(
                            format!("{}", other),
                            "number".to_string(),
                        )),
                    },
                    ValueIteratorOutput::ValuePair(_, _) => Err(StreamError::ReturnTypeError(
                        "value pair".to_string(),
                        "number".to_string(),
                    )),
                    ValueIteratorOutput::Error(err) => {
                        Err(StreamError::IteratorError(format!("{}", err)))
                    }
                })
                .transpose()
        }
    }
}

struct StreamVecDegree {
    value: Option<Vec<Degree>>,
    iterator: Option<ValueIterator>,
}

impl StreamVecDegree {
    fn from_map(map: &DataMap, key: &str, default: f64) -> Result<Self, StreamError> {
        match map.get_with_string(key) {
            Some(value) => StreamVecDegree::from_koto_value(value),
            None => StreamVecDegree::from_koto_value(&Value::Number(default.into())),
        }
    }

    fn process_list(&self, list: ValueList) -> Result<Vec<Degree>, StreamError> {
        list.data()
            .iter()
            .map(|value| match value {
                Value::Number(num) => Ok(Degree::Pitch(f64::from(num))),
                Value::Str(value) => match value.as_str() {
                    "rest" => Ok(Degree::Rest),
                    other => Err(StreamError::ReturnTypeError(
                        format!("{}", other),
                        "number or rest".to_string(),
                    )),
                },
                other => Err(StreamError::ReturnTypeError(
                    format!("{}", other),
                    "number or rest".to_string(),
                )),
            })
            .collect()
    }
}

impl Stream for StreamVecDegree {
    type Item = Vec<Degree>;
    type Error = StreamError;

    fn from_koto_value(value: &Value) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match value {
            Value::Number(num) => Ok(StreamVecDegree {
                value: Some(vec![Degree::Pitch(f64::from(num))]),
                iterator: None,
            }),
            Value::Str(value) => match value.as_str() {
                "rest" => Ok(StreamVecDegree {
                    value: Some(vec![Degree::Rest]),
                    iterator: None,
                }),
                val => Err(StreamError::ValueTypeError(
                    format!("{}", val),
                    "number, iterator or rest".to_string(),
                )),
            },
            Value::Iterator(iterator) => Ok(StreamVecDegree {
                value: None,
                iterator: Some(iterator.clone()),
            }),
            value => Err(StreamError::ValueTypeError(
                format!("{}", value),
                "number, iterator or rest".to_string(),
            )),
        }
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(ref value) = self.value {
            Ok(Some(value.clone()))
        } else {
            // we expect iterator here, if there's no value
            self.iterator
                .as_mut()
                .expect("the iterator is unexpectedly None")
                .next()
                .map(|val| match val {
                    ValueIteratorOutput::Value(value) => match value {
                        Value::Number(num) => Ok(vec![Degree::Pitch(f64::from(num))]),
                        Value::Str(value) => match value.as_str() {
                            "rest" => Ok(vec![Degree::Rest]),
                            other => Err(StreamError::ReturnTypeError(
                                format!("{}", other),
                                "number or rest".to_string(),
                            )),
                        },
                        Value::List(list) => self.process_list(list),
                        other => Err(StreamError::ReturnTypeError(
                            format!("{}", other),
                            "number or rest".to_string(),
                        )),
                    },
                    ValueIteratorOutput::ValuePair(_, _) => Err(StreamError::ReturnTypeError(
                        "value pair".to_string(),
                        "number or rest".to_string(),
                    )),
                    ValueIteratorOutput::Error(err) => {
                        Err(StreamError::IteratorError(format!("{}", err)))
                    }
                })
                .transpose()
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Degree {
    Pitch(f64),
    Rest,
}

impl StreamScale {
    fn from_map(map: &DataMap, key: &str, default: &str) -> Result<Self, StreamError> {
        match map.get_with_string(key) {
            Some(value) => StreamScale::from_koto_value(value),
            None => StreamScale::from_koto_value(&Value::Str(default.into())),
        }
    }
}

struct StreamScale {
    value: Option<Scale>,
    iterator: Option<ValueIterator>,
}

impl Stream for StreamScale {
    type Item = Scale;
    type Error = StreamError;

    fn from_koto_value(value: &Value) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match value {
            Value::Str(value) => Scale::try_from(value.as_str().to_uppercase().as_str())
                .map(|scale| Self {
                    value: Some(scale),
                    iterator: None,
                })
                .map_err(|e| StreamError::OtherError(format!("{}", e))),
            Value::Iterator(iterator) => Ok(Self {
                value: None,
                iterator: Some(iterator.clone()),
            }),
            value => Err(StreamError::ValueTypeError(
                format!("{}", value),
                "number or iterator".to_string(),
            )),
        }
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(value) = self.value {
            Ok(Some(value))
        } else {
            // we expect iterator here, if there's no value
            self.iterator
                .as_mut()
                .expect("the iterator is unexpectedly None")
                .next()
                .map(|val| match val {
                    ValueIteratorOutput::Value(value) => match value {
                        Value::Str(value) => {
                            Scale::try_from(value.as_str().to_uppercase().as_str())
                                .map_err(|e| StreamError::OtherError(format!("{}", e)))
                        }
                        other => Err(StreamError::ReturnTypeError(
                            format!("{}", other),
                            "number".to_string(),
                        )),
                    },
                    ValueIteratorOutput::ValuePair(_, _) => Err(StreamError::ReturnTypeError(
                        "value pair".to_string(),
                        "number".to_string(),
                    )),
                    ValueIteratorOutput::Error(err) => {
                        Err(StreamError::IteratorError(format!("{}", err)))
                    }
                })
                .transpose()
        }
    }
}

#[derive(Default)]
pub(crate) struct Player {
    host: HostCallback,
    parameters: Arc<Parameters>,
    queued_stream: RwLock<Option<ScheduledEventStream>>,
    stream: RwLock<Option<ScheduledEventStream>>,
    wait_until: f64,
    last_position: f64,
}

impl Player {
    pub(crate) fn new(host: HostCallback, parameters: Arc<Parameters>) -> Self {
        Self {
            host,
            parameters,
            ..Default::default()
        }
    }

    fn schedule_stream(&self, e_stream: EventStream, quant: f64) {
        let position = self.position();
        let quant_samples = quant * self.beat_length();
        let offset = quant_samples - (position % quant_samples);
        let position = offset + position;
        *self.queued_stream.write().unwrap() = Some(ScheduledEventStream { position, e_stream });
    }

    fn position(&self) -> f64 {
        self.host.get_time_info(0).unwrap().sample_pos
    }

    fn beat_length(&self) -> f64 {
        let time_info = self
            .host
            .get_time_info(TimeInfoFlags::TEMPO_VALID.bits())
            .unwrap();
        let beats_per_sec = time_info.tempo / 60.0;
        time_info.sample_rate / beats_per_sec
    }

    pub(crate) fn tick(&mut self) -> Option<Vec<MidiEvent>> {
        if !self.is_playing() {
            self.wait_until = self.wait_until - self.last_position;
            return None;
        }

        let position = self.position();

        if position < self.last_position {
            self.wait_until = self.wait_until - self.last_position;
        }

        self.last_position = self.position();

        self.check_queued(self.last_position);

        let result = if let Some(stream) = self.stream.get_mut().unwrap() {
            if position < self.wait_until {
                return None;
            }
            stream
                .e_stream
                .next()
                .map(|event| self.process_events(position, event))
        } else {
            None
        };

        result.map(|event| {
            let beat_length = self.beat_length();
            let length = event.event.length * beat_length * event.event.dur - 1.0;
            event.into_vst_midi(self.host.get_block_size() as f64, length)
        })
    }

    fn is_playing(&self) -> bool {
        if let Some(time_info) = self.host.get_time_info(0) {
            return TimeInfoFlags::from_bits(time_info.flags)
                .map(|val| val.contains(TimeInfoFlags::TRANSPORT_PLAYING))
                .unwrap_or(false);
        }

        false
    }

    /// check if the queued pattern should play
    fn check_queued(&mut self, position: f64) {
        let queued = self.queued_stream.get_mut().unwrap();
        if let Some(stream) = queued.take() {
            if position >= stream.position {
                *self.stream.get_mut().unwrap() = Some(stream);
            } else {
                *queued = Some(stream);
            }
        }
    }

    fn process_events(&mut self, position: f64, event: Event) -> ScheduledEvent {
        let end = event.dur * self.beat_length();
        let offset = position % end;
        self.wait_until = position + end - offset;

        ScheduledEvent { position, event }
    }
}

struct ScheduledEventStream {
    position: f64,
    e_stream: EventStream,
}

struct EventStream {
    iterator: ValueIterator,
    parameters: Arc<Parameters>,
}

impl EventStream {
    fn new(iterator: ValueIterator, parameters: Arc<Parameters>) -> Self {
        EventStream {
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
                    _ => self.post_error(EventStreamError::TypeError),
                },
                ValueIteratorOutput::ValuePair(_, _) => {
                    self.post_error(EventStreamError::TypeError)
                }
                ValueIteratorOutput::Error(err) => {
                    self.parameters.append_console(&format!("{}", err))
                }
            }
        }

        None
    }

    fn event_from_koto(&mut self, koto_event: &ValueMap) -> Result<Event, EventStreamError> {
        let map = koto_event.data();
        let keys = vec!["dur", "length", "channel", "note", "velocity"];
        let values: Result<Vec<&Value>, EventStreamError> = keys
            .iter()
            .map(|k| {
                map.get_with_string(k)
                    .ok_or(EventStreamError::EventTypeError)
            })
            .collect();
        let mut event = HashMap::new();
        values?.into_iter().enumerate().for_each(|(n, val)| {
            event.insert(keys[n], val);
        });

        self.event_from_map(event)
    }

    fn event_from_map(&self, map: HashMap<&str, &Value>) -> Result<Event, EventStreamError> {
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
                _ => return Err(EventStreamError::TypeError),
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
    ) -> Result<Vec<EventValue>, EventStreamError> {
        if self.is_rest(value)? {
            return Ok(vec![EventValue::Rest]);
        }

        let value = match value {
            Value::Number(num) => match num {
                ValueNumber::F64(val) => vec![*val],
                ValueNumber::I64(val) => vec![*val as f64],
            },
            Value::List(list) => self.handle_notes_list(list)?,
            _ => return Err(EventStreamError::TypeError),
        };

        Ok(value
            .iter()
            .map(|nn| EventValue::Note(*nn as u8, velocity, channel))
            .collect())
    }

    fn is_rest(&self, value: &Value) -> Result<bool, EventStreamError> {
        if let Value::Str(val) = value {
            if val.as_str() == "rest" {
                return Ok(true);
            }

            return Err(EventStreamError::TypeError);
        }

        Ok(false)
    }

    fn handle_notes_list(&self, list: &ValueList) -> Result<Vec<f64>, EventStreamError> {
        list.data()
            .iter()
            .map(|val| match val {
                Value::Number(num) => match num {
                    ValueNumber::F64(v) => Ok(*v),
                    ValueNumber::I64(v) => Ok(*v as f64),
                },
                _ => Err(EventStreamError::TypeError),
            })
            .collect()
    }

    fn post_error(&self, error: EventStreamError) {
        self.parameters.append_console(&format!("{}\n", error));
    }
}

impl Iterator for EventStream {
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
enum EventPatternError {
    #[error("Unexpected type for key '{0}', expected type is '{1}")]
    KeyTypeError(String, String),
}

#[derive(Debug, Error)]
enum StreamError {
    #[error("Unexpected value type '{0}' in stream, expected type is '{1}")]
    ValueTypeError(String, String),
    #[error("Unexpected return type '{0}' in stream, expected type is '{1}")]
    ReturnTypeError(String, String),
    #[error("Error processing iterator: '{0}")]
    IteratorError(String),
    #[error("Error in stream: {0}")]
    OtherError(String),
}

#[derive(Debug, Error)]
enum EventStreamError {
    #[error("Unexpected return value type from stream")]
    TypeError,
    #[error("Event type is wrong or incomplete")]
    EventTypeError,
}
