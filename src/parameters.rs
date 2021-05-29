use std::sync::RwLock;

use log::info;

use vst::host::Host;
use vst::plugin::{HostCallback, PluginParameters};

#[derive(Default)]
pub(crate) struct Parameters {
    code: RwLock<String>,
    console_out: RwLock<String>,
    host: Option<HostCallback>,
}

impl Parameters {
    pub(crate) fn set_host(&mut self, host: HostCallback) {
        self.host = Some(host);
    }

    pub(crate) fn set_code(&self, code: &str) {
        if let Some(host) = self.host {
            *self.code.write().unwrap() = code.to_string();
            host.update_display(); // notify host that the plugin changed a parameter
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
}

impl PluginParameters for Parameters {
    fn get_preset_data(&self) -> Vec<u8> {
        info!("getting data");
        self.code.read().unwrap().as_bytes().into()
    }

    fn get_bank_data(&self) -> Vec<u8> {
        self.get_preset_data()
    }

    fn load_preset_data(&self, data: &[u8]) {
        info!("loading state");
        let value = String::from_utf8(data.into()).unwrap();
        *self.code.write().unwrap() = value.clone();
        info!("{}", value)
    }

    fn load_bank_data(&self, data: &[u8]) {
        self.load_preset_data(data);
    }
}
