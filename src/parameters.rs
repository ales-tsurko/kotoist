use std::sync::{Arc, Mutex};

use log::info;

use vst::plugin::PluginParameters;

#[derive(Default)]
pub(crate) struct Parameters {
    pub(crate) code: Arc<Mutex<String>>,
}

impl PluginParameters for Parameters {
    fn get_preset_data(&self) -> Vec<u8> {
        info!("getting data");
        self.code.lock().unwrap().as_bytes().into()
    }

    fn load_preset_data(&self, data: &[u8]) {
        info!("loading state");
        let value = String::from_utf8(data.into()).unwrap();
        *self.code.lock().unwrap() = value.clone();
        info!("{}", value)
    }
}
