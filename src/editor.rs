use std::string::ToString;
use std::sync::{Arc, RwLock};

use vst::editor::{Editor, KeyCode, KnobMode};
use vst_gui::{JavascriptCallback, PluginGui};

use crate::parameters::Parameters;

const HTML: &'static str = include_str!("../gui/build/index.html");
const EDITOR_SIZE: (i32, i32) = (640, 480);

pub(crate) struct KotoistEditor {
    gui: Arc<RwLock<PluginGui>>,
}

impl KotoistEditor {
    pub(crate) fn new(parameters: Arc<Parameters>) -> Self {
        let gui = Arc::new(RwLock::new(vst_gui::new_plugin_gui(
            String::from(HTML),
            make_dispatcher(Arc::clone(&parameters)),
            Some(EDITOR_SIZE),
        )));
        parameters.set_gui(Arc::clone(&gui));
        Self { gui }
    }
}

impl Editor for KotoistEditor {
    fn size(&self) -> (i32, i32) {
        self.gui.read().unwrap().size()
    }

    fn position(&self) -> (i32, i32) {
        self.gui.read().unwrap().position()
    }

    fn open(&mut self, parent: *mut std::ffi::c_void) -> bool {
        self.gui.write().unwrap().open(parent)
    }

    fn is_open(&mut self) -> bool {
        self.gui.write().unwrap().is_open()
    }

    fn idle(&mut self) {
        self.gui.write().unwrap().idle();
    }

    fn close(&mut self) {
        self.gui.write().unwrap().close();
    }

    fn set_knob_mode(&mut self, mode: KnobMode) -> bool {
        self.gui.write().unwrap().set_knob_mode(mode)
    }

    fn key_up(&mut self, keycode: KeyCode) -> bool {
        self.gui.write().unwrap().key_up(keycode)
    }

    fn key_down(&mut self, keycode: KeyCode) -> bool {
        self.gui.write().unwrap().key_down(keycode)
    }
}

fn make_dispatcher(parameters: Arc<Parameters>) -> JavascriptCallback {
    Box::new(move |message: String| -> String {
        let command_str = message.split_whitespace().next().unwrap_or("");
        let command = Command::from(command_str);

        match command {
            Command::SendCode => on_send_code(message, &parameters),
            Command::GetCode => on_get_code(&parameters),
            Command::EvalCode => on_eval_code(message, &parameters),
            Command::SendConsoleOut => on_send_console_out(message, &parameters),
            Command::GetConsoleOut => on_get_console_out(&parameters),
            Command::Unknown => String::new(),
        }
    })
}

fn on_send_code(message: String, parameters: &Arc<Parameters>) -> String {
    let command_str = Command::SendCode.to_string();
    let code = &message[command_str.len() + 1..];
    parameters.set_code(code);
    String::new()
}

fn on_get_code(parameters: &Arc<Parameters>) -> String {
    parameters.code()
}

fn on_eval_code(message: String, parameters: &Arc<Parameters>) -> String {
    let command_str = Command::EvalCode.to_string();
    let code = &message[command_str.len() + 1..];
    parameters.set_code(code);

    "This is sent from Rust\nAnd this is another line from Rust for testing.".to_string()
}

fn on_send_console_out(message: String, parameters: &Arc<Parameters>) -> String {
    let command_str = Command::SendConsoleOut.to_string();
    let out = &message[command_str.len() + 1..];
    parameters.set_console_out(out);
    String::new()
}

fn on_get_console_out(parameters: &Arc<Parameters>) -> String {
    parameters.console_out()
}

#[derive(Debug)]
enum Command {
    SendCode,
    GetCode,
    EvalCode,
    SendConsoleOut,
    GetConsoleOut,
    Unknown,
}

impl From<&str> for Command {
    fn from(message_str: &str) -> Self {
        match message_str {
            "SEND_CODE" => Self::SendCode,
            "GET_CODE" => Self::GetCode,
            "EVAL_CODE" => Self::EvalCode,
            "SEND_CONSOLE_OUT" => Self::SendConsoleOut,
            "GET_CONSOLE_OUT" => Self::GetConsoleOut,
            _ => Self::Unknown,
        }
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        match self {
            Self::SendCode => "SEND_CODE".to_string(),
            Self::GetCode => "GET_CODE".to_string(),
            Self::EvalCode => "EVAL_CODE".to_string(),
            Self::SendConsoleOut => "SEND_CONSOLE_OUT".to_string(),
            Self::GetConsoleOut => "GET_CONSOLE_OUT".to_string(),
            Self::Unknown => String::new(),
        }
    }
}
