use std::sync::RwLock;

use koto::{Koto, KotoSettings};
use vst::host::Host;
use vst::plugin::{HostCallback, PluginParameters};

pub(crate) struct Parameters {
    code: RwLock<String>,
    pub(crate) console_out: RwLock<String>,
    host: Option<HostCallback>,
    pub(crate) is_console_changed: RwLock<bool>,
    pub(crate) koto: RwLock<Koto>,
}

impl Parameters {
    pub(crate) fn set_host(&mut self, host: HostCallback) {
        self.host = Some(host);
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
                Err(err) => self.append_console(&format!("Runtime error: {}\n", err)),
            },
            Err(err) => self.append_console(&format!("Compiler error: {}\n", err)),
        }
    }

    pub(crate) fn append_console(&self, out: &str) {
        let mut console_out = self.console_out.write().unwrap();
        console_out.push_str(out);
        *self.is_console_changed.write().unwrap() = true;
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
            is_console_changed: Default::default(),
            code: Default::default(),
            console_out: Default::default(),
            host: Default::default(),
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
