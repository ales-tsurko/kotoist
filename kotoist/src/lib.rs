//! VST plugin for live coding using [Koto](https://github.com/koto-lang/koto) programming
//! language.

#![deny(nonstandard_style, trivial_casts, trivial_numeric_casts)]
#![warn(
    deprecated_in_future,
    missing_docs,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unreachable_pub
)]
use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;

use crate::editor::create_editor;
use crate::orchestrator::{Event, EventValue};
use crate::parameters::Parameters;

mod editor;
mod interpreter;
mod orchestrator;
mod parameters;
mod pipe;

const NUM_CHANNELS: u32 = 2;

/// Plugin entry-point.
pub struct Kotoist {
    params: Arc<Parameters>,
    editor: Option<Box<dyn Editor>>,
}

impl Default for Kotoist {
    fn default() -> Self {
        let (pipe_in, pipe_out) = pipe::new_pipe();
        let params = Arc::new(Parameters::new(pipe_in));
        let editor = create_editor(params.clone(), Arc::new(Mutex::new(pipe_out)));

        Self { params, editor }
    }
}

impl Plugin for Kotoist {
    const NAME: &'static str = "Kotoist";
    const VENDOR: &'static str = "Ales Tsurko";
    const URL: &'static str = "https://kotoist.alestsurko.by";
    const EMAIL: &'static str = "ales.tsurko@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(NUM_CHANNELS),
        main_output_channels: NonZeroU32::new(NUM_CHANNELS),
        ..AudioIOLayout::const_default()
    }];
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type BackgroundTask = ();
    type SysExMessage = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // as per docs it's:
        // > Queried only once immediately after the plugin instance is created
        // we can do it this way
        self.editor.take()
    }

    // fn process_events(&mut self, events: &Events) {
    //     for e in events.events() {
    //         match e {
    //             EventEnum::Midi(MidiEvent { data, .. }) => {
    //                 if data[0] >= 0x90 || data[0] <= 0x9E && data[2] > 0 {
    //                     self.parameters.eval_snippet_at(data[1] as usize);
    //                     self.parameters.push_event(ParametersEvent {
    //                         command: Command::NoteOn,
    //                         value: format!("{}", data[1]),
    //                     });
    //                 }
    //             }
    //             _ => (),
    //         }
    //     }
    // }

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux: &mut AuxiliaryBuffers<'_>,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Ok(mut orch) = self.params.orchestrator.try_lock() {
            let transport = context.transport();

            orch.tick(transport.playing, &transport.into(), buffer.samples())
                .into_iter()
                .flat_map(|e| plugin_note_from_event(e, buffer.samples()))
                .for_each(|e| context.send_event(e));
        }

        ProcessStatus::KeepAlive
    }
}

impl From<&Transport> for orchestrator::Transport {
    fn from(value: &Transport) -> Self {
        // beats per second tempo
        let tempo = value.tempo.unwrap_or(120.0) / 60.0;
        let position = value.pos_samples().unwrap_or_default() as f64;
        let sample_rate = value.sample_rate as f64;
        let beat_length = sample_rate / tempo;

        orchestrator::Transport {
            position,
            beat_length,
        }
    }
}

fn plugin_note_from_event(event: Event, block_size: usize) -> Vec<PluginNoteEvent<Kotoist>> {
    event
        .value
        .iter()
        .filter_map(|v| match v {
            EventValue::Note(nn, vel, ch) => Some(if *vel > 0 {
                PluginNoteEvent::<Kotoist>::NoteOn {
                    timing: (event.frame_offset % block_size) as u32,
                    channel: *ch,
                    note: *nn,
                    velocity: *vel as f32 / 127.0,
                    voice_id: Some(*nn as i32),
                }
            } else {
                PluginNoteEvent::<Kotoist>::NoteOff {
                    timing: (event.frame_offset % block_size) as u32,
                    channel: *ch,
                    note: *nn,
                    velocity: 0.0,
                    voice_id: Some(*nn as i32),
                }
            }),
            EventValue::Rest => None,
        })
        .collect()
}

impl ClapPlugin for Kotoist {
    const CLAP_ID: &'static str = "by.alestsurko.kotoist";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Live coding using Koto programming language");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for Kotoist {
    const VST3_CLASS_ID: [u8; 16] = *b"KotoistAlesCurko";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Fx,
        Vst3SubCategory::Tools,
    ];
}

// CLAP didn't produce midi outs during testing.
// nih_export_clap!(Kotoist);
nih_export_vst3!(Kotoist);
