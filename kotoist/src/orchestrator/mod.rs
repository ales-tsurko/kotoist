pub(crate) use self::pattern::{Event, EventValue, Pattern, ScheduledEvent};
pub(crate) use self::scale::Scale;

use crate::pipe::{Message as PipeMessage, PipeIn};

mod pattern;
mod scale;

pub(crate) struct Orchestrator {
    pipe_in: PipeIn,
    players: Vec<Player>,
    buffer: Vec<Event>,
}

impl Orchestrator {
    pub(crate) fn new(pipe_in: PipeIn) -> Self {
        Self {
            pipe_in,
            players: Vec::new(),
            buffer: Vec::with_capacity(512),
        }
    }

    pub(crate) fn set_patterns(&mut self, patterns: Vec<Pattern>, quantization: f64) {
        self.players = patterns
            .into_iter()
            .map(|patt| {
                let mut player = self
                    .players
                    .pop()
                    .unwrap_or_else(|| Player::new(self.pipe_in.clone()));
                player.set_pattern(patt, quantization);
                player
            })
            .collect();
    }

    pub(crate) fn tick(
        &mut self,
        is_playing: bool,
        transport: &Transport,
        frame_offset: usize,
    ) -> &[Event] {
        self.buffer.clear();

        self.players
            .iter_mut()
            .flat_map(|p| p.tick(is_playing, transport, frame_offset))
            .for_each(|e| self.buffer.push(e.clone()));

        &self.buffer
    }
}

struct Player {
    pipe_in: PipeIn,
    // 1. user called midiout function and set the pattern with quantization
    requested: Option<Pattern>,
    quantization: f64,
    // 2. the tick is called and the requested pattern scheduled
    scheduled: Option<ScheduledPattern>,
    // 3. the pattern is what should currently play
    stream: Option<ScheduledPattern>,
    next_note_on_pos: f64,
    last_position: f64,
    note_offs: Vec<ScheduledEvent>,
    buffer: Vec<Event>,
}

impl Player {
    fn new(pipe_in: PipeIn) -> Self {
        Player {
            pipe_in,
            requested: None,
            quantization: 0.0,
            scheduled: None,
            stream: None,
            next_note_on_pos: 0.0,
            last_position: 0.0,
            note_offs: Vec::new(),
            buffer: Vec::with_capacity(512),
        }
    }

    fn set_pattern(&mut self, pattern: Pattern, quantization: f64) {
        self.requested = Some(pattern);
        self.quantization = quantization;
    }

    fn tick(&mut self, is_playing: bool, transport: &Transport, frame_offset: usize) -> &[Event] {
        if !is_playing {
            self.next_note_on_pos -= self.last_position;
            self.buffer.clear();
            // return the note offs (if any) to prevent endless tails
            if !self.note_offs.is_empty() {
                self.buffer.append(
                    &mut self
                        .note_offs
                        .clone()
                        .into_iter()
                        .map(|e| e.event)
                        .collect::<Vec<Event>>(),
                );
                self.note_offs.clear();
            }

            return &self.buffer;
        }

        if let Some(pattern) = self.requested.take() {
            let position = quantized_position(self.quantization, transport, frame_offset);
            self.scheduled = Some(ScheduledPattern { position, pattern });
        }

        self.adjust_position(transport, frame_offset);

        self.try_queue(self.last_position);

        self.buffer.clear();

        let mut note_offs = self.note_offs_at(self.last_position);

        self.buffer.append(&mut note_offs);

        if let Some(event) =
            self.next_event(frame_offset, transport.position, transport.beat_length)
        {
            self.buffer.push(event.event);
        }

        &self.buffer
    }

    // adjust next note-on position on cursor jump
    fn adjust_position(&mut self, transport: &Transport, frame_offset: usize) {
        // if the difference is more than two samples, we consider it a jump
        let position = transport.position + frame_offset as f64;
        if (position - self.last_position).abs() > 2.0 {
            self.next_note_on_pos = quantized_position(self.quantization, transport, frame_offset);
            // call note off for all notes
            self.note_offs
                .iter_mut()
                .for_each(|v| v.position = transport.position);
        }

        self.last_position = position;
    }

    fn note_offs_at(&mut self, position: f64) -> Vec<Event> {
        let (current_offs, scheduled_offs) = self
            .note_offs
            .iter()
            .cloned()
            .partition(|v| position >= v.position);

        self.note_offs = scheduled_offs;

        current_offs.into_iter().map(|e| e.event).collect()
    }

    /// try to queue pattern
    fn try_queue(&mut self, position: f64) {
        if let Some(stream) = self.scheduled.take() {
            if position < stream.position {
                self.scheduled = Some(stream);
                return;
            }

            self.stream = Some(stream);

            // the pattern should start playing immediately at the scheduled position. so we need to
            // cut all the playing notes at this position.
            self.next_note_on_pos = if self.next_note_on_pos > position {
                position
            } else {
                self.next_note_on_pos
            };

            // also we need to cut note-offs
            for note_off in self.note_offs.iter_mut() {
                note_off.position = if note_off.position > position {
                    position
                } else {
                    note_off.position
                }
            }
        }
    }

    fn next_event(
        &mut self,
        frame_offset: usize,
        position: f64,
        beat_length: f64,
    ) -> Option<ScheduledEvent> {
        if let Some(stream) = &mut self.stream {
            if position < self.next_note_on_pos {
                return None;
            }

            match stream.pattern.try_next(frame_offset) {
                Ok(event) => return event.map(|e| self.schedule_events(position, beat_length, e)),
                Err(e) => {
                    // we need to remove stream here, as subsequent calls of next will crash Koto
                    self.stream = None;
                    self.pipe_in.send(PipeMessage::Error(format!("{}\n", e)));
                    return None;
                }
            }
        }

        None
    }

    fn schedule_events(&mut self, position: f64, beat_length: f64, event: Event) -> ScheduledEvent {
        let end = event.dur * beat_length;
        self.next_note_on_pos = position + end;
        self.schedule_note_offs(position, beat_length, event.clone());

        ScheduledEvent { position, event }
    }

    fn schedule_note_offs(&mut self, note_on_position: f64, beat_length: f64, mut event: Event) {
        let end = event.length * event.dur * beat_length;
        let position = note_on_position + end;
        event.frame_offset += end as usize;
        event.value.iter_mut().for_each(|e| {
            if let EventValue::Note(_, v, _) = e {
                *v = 0;
            }
        });
        self.note_offs.push(ScheduledEvent { position, event });
    }
}

// get next quantazied position - i.e. the position at which the pattern should play taking the
// quantization into account
fn quantized_position(quantization: f64, transport: &Transport, frame_offset: usize) -> f64 {
    let position = transport.position + frame_offset as f64;
    if quantization == 0.0 {
        return position;
    }
    let quant_samples = quantization * transport.beat_length;
    let offset = quant_samples - (position % quant_samples);
    position + offset
}

#[derive(Debug)]
struct ScheduledPattern {
    position: f64,
    pattern: Pattern,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Transport {
    // in samples
    pub(crate) beat_length: f64,
    // in samples
    pub(crate) position: f64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quantized_position() {
        let sample_rate = 44100.0;
        let bps = 60.0 / 120.0; // beats per second
        let beat_length = bps * sample_rate;
        let mut transport = Transport {
            beat_length,
            position: 1.0,
        };

        assert_eq!(quantized_position(1.0, &transport, 0), beat_length);

        for n in 0..100 {
            transport.position = 42.0 * (n as f64 / 100.0) * sample_rate;
            let quant = 1.5;
            let quant_samples = quant * beat_length;
            assert_eq!(
                quantized_position(quant, &transport, 0) % quant_samples,
                0.0
            );
        }

        transport.position = sample_rate;
        let quant_samples = beat_length; // 1.0 s * beat_length
        assert_eq!(
            quantized_position(1.0, &transport, 0),
            sample_rate + quant_samples
        );
    }
}
