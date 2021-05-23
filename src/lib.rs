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

use std::sync::Once;

use log::{error, info, trace, LevelFilter};

use vst::api::Supported;
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin};
use vst::plugin_main;

#[cfg(debug_assertions)]
static ONCE: Once = Once::new();

#[derive(Default)]
struct Kotoist {
    host: HostCallback,
    sample_rate: f32,
    block_size: i64,
    is_playing: bool,
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

    fn resume(&mut self) {
        info!("hello");
        self.is_playing = true;
    }

    fn suspend(&mut self) {
        info!("suspend");
        self.is_playing = false;
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Kotoist".to_string(),
            vendor: "Ales Tsurko".to_string(),
            unique_id: 4269,
            category: Category::Generator,
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
    simple_logging::log_to_file(path, LevelFilter::Trace).unwrap();
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
    let _ = WriteLogger::init(LevelFilter::Trace, config, file).unwrap();
}

plugin_main!(Kotoist);
