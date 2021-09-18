/// This module contains all editor commands sent between JS GUI and Rust.
///
/// It also contains dispatcher for those commands.

use std::string::ToString;
use std::sync::Arc;

use vst_gui::JavascriptCallback;

use crate::parameters::Parameters;

pub(crate) fn make_dispatcher(parameters: Arc<Parameters>) -> JavascriptCallback {
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

fn on_send_code(message: String, parameters: &Parameters) -> String {
    let command_str = Command::SendCode.to_string();
    let code = &message[command_str.len() + 1..];
    parameters.set_code(code);
    String::new()
}

fn on_get_code(parameters: &Parameters) -> String {
    parameters.code()
}

fn on_eval_code(message: String, parameters: &Parameters) -> String {
    let command_str = Command::EvalCode.to_string();
    let code = &message[command_str.len() + 1..];
    parameters.eval_code(code);
    String::new()
}

fn on_send_console_out(message: String, parameters: &Parameters) -> String {
    let command_str = Command::SendConsoleOut.to_string();
    let out = &message[command_str.len() + 1..];
    parameters.set_console_out(out);
    String::new()
}

fn on_get_console_out(parameters: &Parameters) -> String {
    parameters.console_out()
}

#[derive(Debug)]
pub(crate) enum Command {
    SendCode,
    GetCode,
    EvalCode,
    SendConsoleOut,
    GetConsoleOut,
    Unknown,
}

impl Command {
    pub(crate) fn to_js_event(&self, detail: &str) -> String {
        format!(
            r#"
            (function() {{
                const event = new CustomEvent("{}", {{detail: "{}"}});
                window.dispatchEvent(event);
            }})()
            "#,
            self.to_string(),
            detail.escape_default().to_string(),
        )
    }
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
