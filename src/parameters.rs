use std::fmt::format;
use std::sync::{Arc, RwLock};

use koto::{runtime::Value, Koto};
use log::info;
use vst::host::Host;
use vst::plugin::{HostCallback, PluginParameters};
use vst_gui::PluginGui;

use crate::editor::Command;

#[derive(Default)]
pub(crate) struct Parameters {
    code: RwLock<String>,
    console_out: RwLock<String>,
    host: Option<HostCallback>,
    gui: RwLock<Option<Gui>>,
    koto: RwLock<Koto>,
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

    pub(crate) fn eval_code(&self, code: &str) {
        if let Some(ref gui) = *self.gui.read().unwrap() {
            let mut koto = self.koto.write().unwrap();
            match koto.compile(code) {
                Ok(_) => match koto.run() {
                    Ok(result) => self.append_console(&format!("{}\n", result)),
                    Err(runtime_error) => info!("Runtime error: {}", runtime_error),
                },
                Err(compiler_error) => info!("Compiler error: {}", compiler_error),
            }
        }
    }

    fn append_console(&self, out: &str) {
        let mut console_out = self.console_out.write().unwrap();
        console_out.push_str(out);

        if let Some(ref gui) = *self.gui.read().unwrap() {
            let event = format!(
                r#"
                (function() {{
                    const event = new CustomEvent("{}", {{detail: "{}"}});
                    window.dispatchEvent(event);
                }})()
                "#,
                Command::SendConsoleOut.to_string(),
                console_out.escape_default().to_string(),
            );

            gui.0.read().unwrap().execute(&event).unwrap(); // send response to console
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
