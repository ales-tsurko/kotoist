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
    note_offs: Vec<ScheduledEvent>,
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
        let (position, _) = self.position();
        let offset = quant - (position % quant);
        let position = offset + position;
        *self.queued_pattern.write().unwrap() = Some(ScheduledPattern { position, pattern });
    }

    fn position(&self) -> (f64, f64) {
        // (beat position, sample position)
        let time_info = self
            .host
            .get_time_info(TimeInfoFlags::TEMPO_VALID.bits())
            .unwrap();
        let beats_per_sec = time_info.tempo / 60.0;
        let beat_length = time_info.sample_rate / beats_per_sec;
        (time_info.sample_pos / beat_length, time_info.sample_pos)
    }

    pub(crate) fn process(&mut self) -> Vec<ScheduledEvent> {
        if !self.is_playing() {
            return vec![];
        }

        let (position, sample_pos) = self.position();
        self.check_queued(position);
        let mut result = vec![];

        if let Some(pattern) = self.pattern.get_mut().unwrap() {
            if position >= self.wait_until {
                if let Some(events) = pattern.pattern.next() {
                    let mut sched_events = self.process_events(position, sample_pos, &events);
                    result.append(&mut sched_events);
                }
            }
        }

        let mut note_offs = self.note_offs_at(position);
        result.append(&mut note_offs);
        result
    }

    fn is_playing(&mut self) -> bool {
        if let Some(time_info) = self.host.get_time_info(0) {
            return TimeInfoFlags::from_bits(time_info.flags)
                .map(|val| val.contains(TimeInfoFlags::TRANSPORT_PLAYING))
                .unwrap_or(false);
        }

        false
    }

    /// check if the queued pattern should play
    fn check_queued(&mut self, position: f64) {
        let queued = self.queued_pattern.get_mut().unwrap();
        if let Some(pattern) = queued.take() {
            if pattern.position >= position {
                *self.pattern.get_mut().unwrap() = Some(pattern);
            } else {
                *queued = Some(pattern);
            }
        }
    }

    fn process_events(
        &mut self,
        position: f64,
        sample_pos: f64,
        events: &[Event],
    ) -> Vec<ScheduledEvent> {
        if !events.is_empty() {
            return vec![];
        }

        self.wait_until = position + events[0].dur;

        self.append_note_offs(&events, position, sample_pos);

        events
            .iter()
            .map(|e| ScheduledEvent::On(position, sample_pos as i32, *e))
            .collect()
    }

    fn append_note_offs(&mut self, events: &[Event], position: f64, sample_pos: f64) {
        let note_off_pos = position + events[0].length;

        let mut note_offs: Vec<ScheduledEvent> = events
            .iter()
            .map(|e| ScheduledEvent::Off(note_off_pos, sample_pos as i32, *e))
            .collect();

        self.note_offs.append(&mut note_offs);
    }

    fn note_offs_at(&mut self, position: f64) -> Vec<ScheduledEvent> {
        // return (and remove from the list) note offs which valid for current position
        let (result, sched): (Vec<ScheduledEvent>, Vec<ScheduledEvent>) =
            self.note_offs.iter().partition(|e| {
                if let ScheduledEvent::Off(p, _, _) = e {
                    *p >= position
                } else {
                    false
                }
            });
        self.note_offs = sched;
        return result;
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

        let value = if is_rest {
            EventValue::Rest
        } else {
            EventValue::Note(
                event["note"] as u8,
                event["velocity"] as u8,
                event["channel"] as u8,
            )
        };

        Ok(Event {
            value,
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
        self.parameters.append_console(&format!("{}\n", error));
    }
}

impl Iterator for Pattern {
    type Item = Vec<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iterator = self.iterator.clone();
        self.next_in_iterator(&mut iterator)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ScheduledEvent {
    On(f64, i32, Event), // beat position, sample position, event
    Off(f64, i32, Event),
}

impl ScheduledEvent {
    pub(crate) fn into_vst_midi(self, block_size: i32) -> Option<MidiEvent> {
        let (pos, midi_data) = match self {
            Self::On(_, sp, e) => match e.value {
                EventValue::Rest => return None,
                EventValue::Note(nn, vel, ch) => (sp, [0x90 + ch, nn, vel]),
            },
            Self::Off(_, sp, e) => match e.value {
                EventValue::Rest => return None,
                EventValue::Note(nn, vel, ch) => (sp, [0x80 + ch, nn, vel]),
            },
        };

        let delta_frames = pos % block_size;

        Some(MidiEvent {
            event_type: EventType::Midi,
            byte_size: 8,
            delta_frames,
            flags: MidiEventFlags::REALTIME_EVENT.bits(),
            note_length: 0,
            note_offset: 0,
            midi_data,
            _midi_reserved: 0,
            detune: 0,
            note_off_velocity: 0,
            _reserved1: 0,
            _reserved2: 0,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Event {
    pub(crate) value: EventValue,
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
