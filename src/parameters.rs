use std::sync::{Arc, RwLock};

use koto::runtime::ValueMap;
use koto::{Koto, KotoSettings};
use log::info;
use vst::host::Host;
use vst::plugin::{HostCallback, PluginParameters};
use vst_gui::PluginGui;

use crate::command::Command;

// #[derive(Default)]
pub(crate) struct Parameters {
    code: RwLock<String>,
    console_out: RwLock<String>,
    host: Option<HostCallback>,
    gui: RwLock<Option<Gui>>,
    pub(crate) koto: RwLock<Koto>,
}

impl Parameters {
    pub(crate) fn set_host(&mut self, host: HostCallback) {
        self.host = Some(host);
    }

    pub(crate) fn set_gui(&self, gui: Arc<RwLock<PluginGui>>) {
        *self.gui.write().unwrap() = Some(Gui(gui));
    }

    pub(crate) fn set_code(&self, code: &str) {
        if let Some(host) = self.host {
            *self.code.write().unwrap() = code.to_string();
            host.update_display(); // notify host that the plugin is changing a parameter
        }
    }

    pub(crate) fn code(&self) -> String {
        (*self.code.read().unwrap()).clone()
    }

    pub(crate) fn set_console_out(&self, out: &str) {
        *self.console_out.write().unwrap() = out.to_string();
    }

    pub(crate) fn console_out(&self) -> String {
        (*self.console_out.read().unwrap()).clone()
    }

    pub(crate) fn eval_code(&self, code: &str) {
        let mut koto = self.koto.write().unwrap();
        match koto.compile(code) {
            Ok(_) => match koto.run() {
                Ok(result) => self.append_console(&format!("{}\n", result)),
                Err(err) => self.append_console(&format!("Runtime error: {}", err)),
            },
            Err(err) => self.append_console(&format!("Compiler error: {}", err)),
        }
    }

    fn append_console(&self, out: &str) {
        let mut console_out = self.console_out.write().unwrap();
        console_out.push_str(out);

        if let Some(ref gui) = *self.gui.read().unwrap() {
            // send response to console
            gui.0
                .read()
                .unwrap()
                .execute(&Command::SendConsoleOut.to_js_event(&console_out))
                .unwrap();
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        let koto = Koto::with_settings(KotoSettings {
            repl_mode: true,
            ..Default::default()
        });

        // ..Default::default() calls it recursively, so we call it for each field separatelly
        Self {
            koto: RwLock::new(koto),
            code: Default::default(),
            console_out: Default::default(),
            host: Default::default(),
            gui: Default::default(),
        }
    }
}

impl PluginParameters for Parameters {
    fn get_preset_data(&self) -> Vec<u8> {
        self.code.read().unwrap().as_bytes().into()
    }

    fn get_bank_data(&self) -> Vec<u8> {
        self.get_preset_data()
    }

    fn load_preset_data(&self, data: &[u8]) {
        let value = String::from_utf8(data.into()).unwrap();
        *self.code.write().unwrap() = value.clone();
    }

    fn load_bank_data(&self, data: &[u8]) {
        self.load_preset_data(data);
    }
}

struct Gui(Arc<RwLock<PluginGui>>);

unsafe impl Send for Gui {}
unsafe impl Sync for Gui {}
