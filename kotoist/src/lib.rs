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


use nih_plug::prelude::*;
use std::sync::{Arc, Mutex, Once};

use log::{info, LevelFilter};

// use vst::api::{Event, Events, Supported};
// use vst::event::{Event as EventEnum, MidiEvent};
// use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin, PluginParameters};
// use vst::{buffer::AudioBuffer, editor::Editor, host::Host, plugin_main};

// use crate::editor::{command::Command, KotoistEditor};
// use crate::parameters::{Event as ParametersEvent, Parameters};
// use crate::orchestrator::{make_module, Orchestrator};

// mod editor;
mod parameters;
mod orchestrator;
mod interpreter;
mod pipe;


#[cfg(debug_assertions)]
static ONCE: Once = Once::new();

#[derive(Default)]
pub struct Kotoist {
    // host: HostCallback,
    sample_rate: f32,
    block_size: i64,
    // parameters: Arc<Parameters>,
    // orchestrator: Arc<Mutex<Orchestrator>>,
}

impl Plugin for Kotoist {
    const NAME: &'static str = "Kotoist";
    const VENDOR: &'static str = "Ales Tsurko";
    const URL: &'static str = "https://kotoist.alestsurko.by";
    const EMAIL: &'static str = "ales.tsurko@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type BackgroundTask = ();
    type SysExMessage = ();

    fn params(&self) -> Arc<dyn Params> {
        todo!()
    }

    // fn new(host: HostCallback) -> Self {
    //     #[cfg(debug_assertions)]
    //     init_log();

    //     let mut parameters = Parameters::default();
    //     parameters.set_host(host.clone());
    //     let parameters = Arc::new(parameters);
    //     let orchestrator = Arc::new(Mutex::new(Orchestrator::new(
    //         host.clone(),
    //         Arc::clone(&parameters),
    //     )));
    //     let mut prelude = parameters.koto.write().unwrap().prelude();
    //     prelude.add_map("pattern", make_module(Arc::clone(&orchestrator)));
    //     prelude.add_value("random", make_random_module());

    //     parameters.eval_code("from pattern import midi_out, print_scales");
    //     parameters.eval_code(KOTO_LIB_CODE);

    //     Self {
    //         host,
    //         parameters,
    //         orchestrator,
    //         ..Default::default()
    //     }
    // }

    // fn set_sample_rate(&mut self, rate: f32) {
    //     self.sample_rate = rate;
    // }

    // fn set_block_size(&mut self, size: i64) {
    //     self.block_size = size;
    // }

    // fn can_do(&self, can_do: CanDo) -> Supported {
    //     use CanDo::*;
    //     use Supported::*;

    //     match can_do {
    //         SendEvents
    //         | SendMidiEvent
    //         | ReceiveEvents
    //         | ReceiveMidiEvent
    //         | ReceiveTimeInfo
    //         | MidiProgramNames
    //         | Bypass
    //         | ReceiveSysExEvent
    //         | MidiSingleNoteTuningChange
    //         | MidiKeyBasedInstrumentControl => Yes,
    //         _ => Maybe,
    //     }
    // }

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
        aux: &mut AuxiliaryBuffers<'_>,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        todo!()
    }

    // fn process(
    //     &mut self,
    //     _buffer: &mut AudioBuffer<'_, f32>,
    //     _aux: &mut AuxiliaryBuffers<'_>,
    //     _context: &mut impl ProcessContext<Self>,
    // ) -> ProcessStatus {
    //     // FIXME: oh no
    //     let events = self.orchestrator.lock().unwrap().tick();
    //     for mut event in events.into_iter() {
    //         let conv: *mut Event = unsafe { std::mem::transmute(&mut event) };
    //         let events = Events {
    //             num_events: 1,
    //             _reserved: 0,
    //             events: [conv, conv],
    //         };
    //         self.host.process_events(&events);
    //     }

    //     ProcessStatus::KeepAlive
    // }

    // fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
    //     Some(Box::new(KotoistEditor::new(Arc::clone(&self.parameters))))
    // }

    // fn params(&mut self) -> Arc<dyn Params> {
    //     let result = Arc::clone(&self.parameters);
    //     let result: Arc<dyn Params> = result;
    //     result
    // }
}

// #[cfg(debug_assertions)]
// fn init_log() {
//     ONCE.call_once(|| {
//         _init_log(LevelFilter::Debug);
//         info!("init log");
//     });
// }

// #[cfg(windows)]
// fn _init_log(level: LevelFilter) {
//     use simple_logging;
//     let path = format!("{}/Desktop/kotoist.log", std::env::var("HOMEPATH").unwrap());
//     simple_logging::log_to_file(path, level).unwrap();
// }

// #[cfg(unix)]
// fn _init_log(level: LevelFilter) {
//     use simplelog::{ConfigBuilder, WriteLogger};
//     use std::fs::OpenOptions;
//     let path = format!("{}/Desktop/kotoist.log", std::env::var("HOME").unwrap());

//     let file = OpenOptions::new()
//         .append(true)
//         .create(true)
//         .open(path)
//         .unwrap();
//     let config = ConfigBuilder::new().set_time_to_local(true).build();
//     let _ = WriteLogger::init(level, config, file).unwrap();
// }

// #[allow(missing_docs)]
// plugin_main!(Kotoist);
