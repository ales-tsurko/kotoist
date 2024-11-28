pub(crate) use self::pattern::{Event, EventValue, Pattern, ScheduledEvent};
pub(crate) use self::scale::Scale;

use crate::pipe::{Message as PipeMessage, PipeIn};

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
    requested: Option<Pattern>,
    quantization: f64,
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
            quantization: 0.0,
            scheduled: None,
            stream: None,
            next_note_on_pos: 0.0,
            last_position: 0.0,
            note_offs: Vec::new(),
        }
    }

    fn set_pattern(&mut self, pattern: Pattern, quantization: f64) {
        self.requested = Some(pattern);
        self.quantization = quantization;
    }

    fn tick(&mut self, is_playing: bool, transport: &Transport, block_size: usize) -> Vec<Event> {
        if !is_playing {
            self.next_note_on_pos -= self.last_position;
            // return the note offs (if any) to prevent endless tails
            // after that just clone and return an empty vector (because of .clear())
            let res = self
                .note_offs
                .clone()
                .into_iter()
                .map(|e| e.event)
                .collect();
            self.note_offs.clear();

            return res;
        }

        if let Some(pattern) = self.requested.take() {
            self.scheduled = Some(ScheduledPattern {
                position: self.quantized_position(transport),
                pattern,
            });
        }

        self.adjust_position(transport, block_size);

        self.try_queue(self.last_position);

        let mut result = vec![];

        for n in 0..block_size {
            result.append(&mut self.note_offs_at(self.last_position + n as f64));
            if let Some(event) = self.next_event(n, transport.position, transport.beat_length) {
                result.push(event.event);
            }
        }

        result
    }

    // get next quantazied position - i.e. the position at which the pattern should play taking the
    // quantization into account
    fn quantized_position(&self, transport: &Transport) -> f64 {
        if self.quantization == 0.0 {
            return transport.position;
        }
        let quant_samples = self.quantization * transport.beat_length;
        let offset = quant_samples - (transport.position % quant_samples);
        transport.position + offset
    }

    // adjust next note on position on cursor jump
    fn adjust_position(&mut self, transport: &Transport, block_size: usize) {
        // if the difference is more than block_size + a sample, we consider it a jump
        if (transport.position - self.last_position).abs() > (block_size + 1) as f64 {
            self.next_note_on_pos = self.quantized_position(transport);
            // call note off for all on the next sample
            self.note_offs
                .iter_mut()
                .for_each(|v| v.position = transport.position + 1.0);
        }

        self.last_position = transport.position;
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
        frame_offset: usize,
        position: f64,
        beat_length: f64,
    ) -> Option<ScheduledEvent> {
        let position = position + frame_offset as f64;

        if let Some(stream) = &mut self.stream {
            if position < self.next_note_on_pos {
                return None;
            }

            match stream.pattern.try_next(frame_offset) {
                Ok(event) => return event.map(|e| self.schedule_events(position, beat_length, e)),
                Err(e) => {
                    // we need to remove stream here as subsequent calls of next, will crash Koto
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

#[derive(Debug)]
struct ScheduledPattern {
    position: f64,
    pattern: Pattern,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Transport {
    pub(crate) beat_length: f64,
    pub(crate) position: f64,
}
