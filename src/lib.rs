//! VST plugin for live coding using [Koto](https://github.com/koto-lang/koto) programming
//! language.

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts
)]
#![warn(
    deprecated_in_future,
    missing_docs,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unreachable_pub
)]

mod editor;
mod parameters;

use std::sync::{Arc, Once};

use log::{info, LevelFilter};

use vst::api::{Event, EventType, Events, MidiEvent, Supported, TimeInfoFlags};
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin, PluginParameters};
use vst::{buffer::AudioBuffer, editor::Editor, host::Host, plugin_main};

use editor::KotoistEditor;
use parameters::Parameters;

#[cfg(debug_assertions)]
static ONCE: Once = Once::new();

#[derive(Default)]
struct Kotoist {
    host: HostCallback,
    sample_rate: f32,
    block_size: i64,
    is_playing: bool,
    count: i64,
    parameters: Arc<Parameters>,
}

impl Kotoist {
    fn update_play_state(&mut self) {
        if let Some(time_info) = self.host.get_time_info(0) {
            self.is_playing = TimeInfoFlags::from_bits(time_info.flags)
                .map(|val| val.contains(TimeInfoFlags::TRANSPORT_PLAYING))
                .unwrap_or(false);
        }
    }
}

impl Plugin for Kotoist {
    fn new(host: HostCallback) -> Self {
        Self {
            host,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        #[cfg(debug_assertions)]
        init_log();
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }

    fn set_block_size(&mut self, size: i64) {
        self.block_size = size;
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Kotoist".to_string(),
            vendor: "Ales Tsurko".to_string(),
            unique_id: 4269,
            category: Category::Generator,
            // preset_chunks: true,
            parameters: 1,
            ..Default::default()
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        use CanDo::*;
        use Supported::*;

        match can_do {
            SendEvents => Yes,
            SendMidiEvent => Yes,
            ReceiveEvents => Yes,
            ReceiveMidiEvent => Yes,
            ReceiveTimeInfo => Yes,
            Offline => Maybe,
            MidiProgramNames => Yes,
            Bypass => Yes,
            ReceiveSysExEvent => Yes,
            MidiSingleNoteTuningChange => Yes,
            MidiKeyBasedInstrumentControl => Yes,
            Other(_) => Maybe,
        }
    }

    fn process(&mut self, _buffer: &mut AudioBuffer<'_, f32>) {
        self.update_play_state();

        if self.is_playing {
            if self.count % 50 == 0 {
                let mut event = MidiEvent {
                    event_type: EventType::Midi,
                    byte_size: 8,
                    delta_frames: 0,
                    flags: 0,
                    note_length: 1000,
                    note_offset: 0,
                    midi_data: [0x9c, 60, 100],
                    _midi_reserved: 0,
                    detune: 0,
                    note_off_velocity: 0,
                    _reserved1: 0,
                    _reserved2: 0,
                };
                let conv: *mut Event = unsafe { std::mem::transmute(&mut event) };
                let events = Events {
                    num_events: 1,
                    _reserved: 0,
                    events: [conv, conv],
                };
                self.host.process_events(&events);
            }
            self.count += 1;
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        Some(KotoistEditor::new(Arc::clone(&self.parameters)).into_handler())
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        let result = Arc::clone(&self.parameters);
        let result: Arc<dyn PluginParameters> = result;
        result
    }
}

#[cfg(debug_assertions)]
fn init_log() {
    ONCE.call_once(|| {
        let path = format!("{}/Desktop/kotoist.log", std::env::var("HOME").unwrap());
        _init_log(path);
        info!("init log");
    });
}

#[cfg(windows)]
fn _init_log(path: String) {
    use simple_logging;
    simple_logging::log_to_file(path, LevelFilter::Info).unwrap();
}

#[cfg(unix)]
fn _init_log(path: String) {
    use simplelog::{ConfigBuilder, WriteLogger};
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let config = ConfigBuilder::new().set_time_to_local(true).build();
    let _ = WriteLogger::init(LevelFilter::Info, config, file).unwrap();
}

#[allow(missing_docs)]
plugin_main!(Kotoist);
