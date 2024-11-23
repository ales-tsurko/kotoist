use std::convert::TryFrom;

use koto::runtime::KMap;

use super::stream::*;
use crate::orchestrator::Scale;

#[derive(Debug)]
pub(crate) struct Pattern {
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

impl Pattern {
    pub(crate) fn try_next(&mut self, frame_offset: usize) -> Result<Option<Event>, Error> {
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

        Ok(Some(Event {
            value,
            dur,
            length,
            frame_offset,
        }))
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

impl TryFrom<&KMap> for Pattern {
    type Error = Error;

    fn try_from(map: &KMap) -> Result<Self, Self::Error> {
        let dur = StreamF64::from_map(map, "dur", 1.0)?;
        let length = StreamF64::from_map(map, "length", 1.0)?;
        let degree = StreamVecDegree::from_map(map, "degree", 0.0)?;
        let scale = StreamScale::from_map(map, "scale", "chromatic")?;
        let root = StreamF64::from_map(map, "root", 0.0)?;
        let transpose = StreamF64::from_map(map, "transpose", 0.0)?;
        let mtranspose = StreamF64::from_map(map, "mtranspose", 0.0)?;
        let octave = StreamF64::from_map(map, "octave", 5.0)?;
        let channel = StreamF64::from_map(map, "channel", 0.0)?;
        let amp = StreamF64::from_map(map, "amp", 0.85)?;

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

#[derive(Debug, Clone)]
pub(crate) struct ScheduledEvent {
    pub(crate) position: f64,
    pub(crate) event: Event,
}

// impl ScheduledEvent {
//     pub(crate) fn into_vst_midi(self, length: f64) -> Vec<MidiEvent> {
//         self.event
//             .value
//             .iter()
//             .filter_map(|value| match value {
//                 EventValue::Note(nn, vel, ch) => {
//                     let status = if *vel > 0 { 0x90 } else { 0x80 };
//                     Some([status + ch, *nn, *vel])
//                 }
//                 &EventValue::Rest => None,
//             })
//             .map(|midi_data| MidiEvent {
//                 event_type: EventType::Midi,
//                 byte_size: 8,
//                 delta_frames: 0,
//                 flags: MidiEventFlags::REALTIME_EVENT.bits(),
//                 note_length: length as i32,
//                 note_offset: 0,
//                 midi_data,
//                 _midi_reserved: 0,
//                 detune: 0,
//                 note_off_velocity: 0,
//                 _reserved1: 0,
//                 _reserved2: 0,
//             })
//             .collect()
//     }
// }

#[derive(Debug, Clone)]
pub(crate) struct Event {
    pub(crate) value: Vec<EventValue>,
    /// sample position within the block
    pub(crate) frame_offset: usize,
    pub(crate) dur: f64,
    pub(crate) length: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum EventValue {
    // note number, velocity, channel number
    // velocity == 0 is note-off
    Note(u8, u8, u8),
    Rest,
}
