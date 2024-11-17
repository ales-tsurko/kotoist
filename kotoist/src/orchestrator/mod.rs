use std::sync::mpsc::{self, Receiver, Sender};

pub(crate) use self::pattern::{Event, EventValue, Pattern, ScheduledEvent};
pub(crate) use self::scale::Scale;

use crate::pipe::PipeIn;

mod pattern;
mod scale;

pub(crate) struct Orchestrator {
    pipe_in: PipeIn,
    players: Vec<Player>,
}

impl Orchestrator {
    pub(crate) fn new(pipe_in: PipeIn) -> Self {
        Self {
            pipe_in,
            players: Vec::new(),
        }
    }

    pub(crate) fn set_patterns(&mut self, patterns: Vec<Pattern>, quantization: f64) {
        self.players = patterns
            .into_iter()
            .map(|patt| match self.players.pop() {
                Some(mut player) => {
                    player.set_pattern(patt, quantization);
                    player
                }
                None => {
                    let mut player = Player::new(self.pipe_in.clone());
                    player.set_pattern(patt, quantization);
                    player
                }
            })
            .collect();
    }

    pub(crate) fn tick(
        &mut self,
        is_playing: bool,
        transport: &Transport,
        block_size: usize,
    ) -> Vec<Event> {
        self.players
            .iter_mut()
            .flat_map(|p| p.tick(is_playing, transport, block_size))
            .collect()
    }
}

struct Player {
    pipe_in: PipeIn,
    // 1. user called midi_out function and set the pattern with quantization
    requested: Option<(Pattern, f64)>,
    // 2. the tick is called and the requested pattern scheduled
    scheduled: Option<ScheduledPattern>,
    // 3. the pattern is what should currently play
    stream: Option<ScheduledPattern>,
    next_note_on_pos: f64,
    last_position: f64,
    note_offs: Vec<ScheduledEvent>,
}

impl Player {
    fn new(pipe_in: PipeIn) -> Self {
        Player {
            pipe_in,
            requested: None,
            scheduled: None,
            stream: None,
            next_note_on_pos: 0.0,
            last_position: 0.0,
            note_offs: Vec::new(),
        }
    }

    fn set_pattern(&mut self, pattern: Pattern, quantization: f64) {
        self.requested = Some((pattern, quantization));
    }

    fn tick(&mut self, is_playing: bool, transport: &Transport, block_size: usize) -> Vec<Event> {
        if !is_playing {
            self.next_note_on_pos -= self.last_position;
            self.note_offs.clear();
            return vec![];
        }

        if let Some((pattern, quantization)) = self.requested.take() {
            self.schedule_pattern(pattern, transport, quantization);
        }

        self.adjust_position(transport.position);

        self.try_queue(self.last_position);

        let mut result = vec![];

        for n in 0..block_size {
            result.append(&mut self.note_offs_at(self.last_position + n as f64));
            if let Some(event) =
                self.next_event(n as f64, transport.position, transport.beat_length)
            {
                result.push(event.event);
            }
        }

        result
    }

    fn schedule_pattern(&mut self, pattern: Pattern, transport: &Transport, quantization: f64) {
        let position = transport.position;
        let quant_samples = quantization * transport.beat_length;
        let offset = quant_samples - (position % quant_samples);
        let position = offset + position;
        self.scheduled = Some(ScheduledPattern { position, pattern });
    }

    fn adjust_position(&mut self, position: f64) {
        if position < self.last_position {
            let last_position = self.last_position;
            self.next_note_on_pos -= last_position;
            self.note_offs
                .iter_mut()
                .for_each(|v| v.position -= last_position);
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
            if position >= stream.position {
                self.stream = Some(stream);
            } else {
                self.scheduled = Some(stream);
            }
        }
    }

    fn next_event(
        &mut self,
        frame_offset: f64,
        position: f64,
        beat_length: f64,
    ) -> Option<ScheduledEvent> {
        let position = position + frame_offset;

        if let Some(stream) = &mut self.stream {
            if position < self.next_note_on_pos {
                return None;
            }

            match stream.pattern.try_next() {
                Ok(event) => return event.map(|e| self.schedule_events(position, beat_length, e)),
                Err(e) => {
                    self.pipe_in.send_err(&format!("{}\n", e));
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
        event.value.iter_mut().for_each(|e| {
            if let EventValue::Note(_, v, _) = e {
                *v = 0;
            }
        });
        self.note_offs.push(ScheduledEvent { position, event });
    }
}

struct ScheduledPattern {
    position: f64,
    pattern: Pattern,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Transport {
    pub(crate) beat_length: f64,
    pub(crate) position: f64,
}
