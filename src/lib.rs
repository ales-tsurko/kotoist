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

use vst::api::Supported;
use vst::plugin::{CanDo, Category, Info, Plugin};
use vst::plugin_main;

#[derive(Default)]
struct Kotoist;

impl Plugin for Kotoist {
    fn get_info(&self) -> Info {
        Info {
            name: "Kotoist".to_string(),
            vendor: "Ales Tsurko".to_string(),
            unique_id: 2742,
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

plugin_main!(Kotoist);
