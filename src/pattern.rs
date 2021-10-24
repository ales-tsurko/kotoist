mod scale;

use std::convert::TryFrom;
use std::sync::{Arc, Mutex, RwLock};

use koto::runtime::{
    runtime_error, DataMap, Value, ValueIterator, ValueIteratorOutput, ValueList, ValueMap,
};
use thiserror::Error;
use vst::api::{EventType, MidiEvent, MidiEventFlags, TimeInfoFlags};
use vst::host::Host;
use vst::plugin::HostCallback;

use crate::parameters::Parameters;
use scale::Scale;

pub(crate) fn make_module(orchestrator: Arc<Mutex<Orchestrator>>) -> ValueMap {
    use Value::{Empty, List, Map, Number};

    let mut result = ValueMap::new();

    let orchestrator_cl = orchestrator.clone();

    result.add_fn("midi_out", {
        move |vm, args| match vm.get_args(args) {
            [Map(map), Number(quant)] => {
                let quant = f64::from(quant);

                match EventPattern::try_from(map) {
                    Ok(pattern) => {
                        orchestrator_cl
                            .lock()
                            .unwrap()
                            .schedule_patterns(vec![pattern], quant);
                    }
                    Err(e) => return runtime_error!("{}", e),
                }

                Ok(Empty)
            }
            [List(list), Number(quant)] => {
                let quant = f64::from(quant);
                let mut patterns = Vec::new();

                for item in list.clone().data().iter() {
                    match item {
                        Map(map) => match EventPattern::try_from(map) {
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

                orchestrator_cl
                    .lock()
                    .unwrap()
                    .schedule_patterns(patterns, quant);

                Ok(Empty)
            }
            _ => runtime_error!(
                "pattern.midi_out: \
                Expected arguments: map or list of maps, quantization."
            ),
        }
    });

    let orchestrator_cl = orchestrator.clone();

    result.add_fn("print_scales", {
        move |vm, args| match vm.get_args(args) {
            [] => {
                orchestrator_cl
                    .lock()
                    .unwrap()
                    .parameters
                    .post_stdout(&Scale::list());
                Ok(Empty)
            }
            _ => runtime_error!("pattern.print_scales: doesn't expect any arguments"),
        }
    });

    result
}

#[derive(Default)]
pub(crate) struct Orchestrator {
    host: HostCallback,
    parameters: Arc<Parameters>,
    players: Vec<Player>,
}

impl Orchestrator {
    pub(crate) fn new(host: HostCallback, parameters: Arc<Parameters>) -> Self {
        Self {
            host,
            parameters,
            ..Default::default()
        }
    }

    fn schedule_patterns(&mut self, patterns: Vec<EventPattern>, quant: f64) {
        self.players = patterns
            .into_iter()
            .map(|p| match self.players.pop() {
                Some(player) => {
                    player.schedule_pattern(p, quant);
                    player
                }
                None => {
                    let player = Player::new(self.host.clone(), Arc::clone(&self.parameters));
                    player.schedule_pattern(p, quant);
                    player
                }
            })
            .collect();
    }

    pub(crate) fn tick(&mut self) -> Vec<MidiEvent> {
        self.players
            .iter_mut()
            .filter_map(Player::tick)
            .flatten()
            .collect()
    }
}

#[derive(Default)]
struct Player {
    host: HostCallback,
    parameters: Arc<Parameters>,
    queued_stream: RwLock<Option<ScheduledEventPattern>>,
    stream: RwLock<Option<ScheduledEventPattern>>,
    next_note_on_pos: f64,
    last_position: f64,
    note_offs: Vec<ScheduledEvent>,
}

impl Player {
    fn new(host: HostCallback, parameters: Arc<Parameters>) -> Self {
        Self {
            host,
            parameters,
            ..Default::default()
        }
    }

    fn schedule_pattern(&self, pattern: EventPattern, quant: f64) {
        let position = self.position();
        let quant_samples = quant * self.beat_length();
        let offset = quant_samples - (position % quant_samples);
        let position = offset + position;
        *self.queued_stream.write().unwrap() = Some(ScheduledEventPattern { position, pattern });
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

    fn tick(&mut self) -> Option<Vec<MidiEvent>> {
        if !self.is_playing() {
            self.next_note_on_pos -= self.last_position;
            self.note_offs.clear();
            return None;
        }

        self.adjust_cursor_jump();

        let mut note_offs = self.note_offs_at(self.last_position);

        self.check_queued(self.last_position);

        if let Some(event) = self.next_event() {
            let beat_length = self.beat_length();
            let length = event.event.length * beat_length * event.event.dur - 1.0;
            let mut result = event.into_vst_midi(self.host.get_block_size() as f64, length);
            note_offs.append(&mut result);
        };

        Some(note_offs)
    }

    fn is_playing(&self) -> bool {
        if let Some(time_info) = self
            .host
            .get_time_info((TimeInfoFlags::TRANSPORT_PLAYING).bits())
        {
            return TimeInfoFlags::from_bits(time_info.flags)
                .map(|val| val.contains(TimeInfoFlags::TRANSPORT_PLAYING))
                .unwrap_or(false);
        }

        false
    }

    fn adjust_cursor_jump(&mut self) {
        let position = self.position();

        if position < self.last_position {
            let last_position = self.last_position;
            self.next_note_on_pos -= last_position;
            self.note_offs
                .iter_mut()
                .for_each(|v| v.position -= last_position);
        }

        self.last_position = position;
    }

    fn note_offs_at(&mut self, position: f64) -> Vec<MidiEvent> {
        let (current_offs, scheduled_offs) = self
            .note_offs
            .iter()
            .cloned()
            .partition(|v| position >= v.position);

        self.note_offs = scheduled_offs;

        current_offs
            .into_iter()
            .map(|e| e.into_vst_midi(self.host.get_block_size() as f64, 1.0))
            .flatten()
            .collect()
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

    fn next_event(&mut self) -> Option<ScheduledEvent> {
        let position = self.position();

        if let Some(stream) = self.stream.get_mut().unwrap() {
            if position < self.next_note_on_pos {
                return None;
            }

            match stream.pattern.try_next() {
                Ok(event) => return event.map(|e| self.schedule_events(position, e)),
                Err(e) => {
                    self.parameters.post_stderr(&format!("{}\n", e));
                    return None;
                }
            }
        }

        None
    }

    fn schedule_events(&mut self, position: f64, event: Event) -> ScheduledEvent {
        let end = event.dur * self.beat_length();
        let offset = position % end;
        self.next_note_on_pos = position + end - offset;
        self.schedule_note_offs(position, event.clone());

        ScheduledEvent { position, event }
    }

    fn schedule_note_offs(&mut self, note_on_position: f64, mut event: Event) {
        let end = event.length * event.dur * self.beat_length();
        let position = note_on_position + end;
        event.value.iter_mut().for_each(|e| match e {
            EventValue::Note(_, v, _) => *v = 0,
            _ => (),
        });
        self.note_offs.push(ScheduledEvent { position, event });
    }
}

struct ScheduledEventPattern {
    position: f64,
    pattern: EventPattern,
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

        let velocity = (127.0 * amp).clamp(0.0, 127.0) as u8;

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
        let octave = (12.0 * octave).clamp(0.0, 120.0);
        let root = root + octave + transpose;

        degree
            .iter()
            .map(|d| match d {
                Degree::Pitch(p) => {
                    let pitch = mtranspose + p;
                    let ps_len = pitch_set.len() as f64;
                    let is_neg = pitch.is_sign_negative() as u8;
                    let oct = ((pitch + is_neg as f64) / ps_len) as i16 - is_neg as i16;
                    let pitch =
                        (oct.abs() as f64 * 2.0 * ps_len + pitch) as usize % pitch_set.len();
                    let oct = oct as f64 * 12.0;
                    Degree::Pitch(pitch_set[pitch] + root + oct)
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
                EventValue::Note(nn, vel, ch) => {
                    let status = if *vel > 0 { 0x90 } else { 0x80 };
                    Some([status + ch, *nn, *vel])
                }
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
enum StreamError {
    #[error("Unexpected value type '{0}' in stream, expected type is {1}")]
    ValueTypeError(String, String),
    #[error("Unexpected return type '{0}' in stream, expected type is {1}")]
    ReturnTypeError(String, String),
    #[error("Error processing iterator: '{0}'")]
    IteratorError(String),
    #[error("Error in stream: {0}")]
    OtherError(String),
}
