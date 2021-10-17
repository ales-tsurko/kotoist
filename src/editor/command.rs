/// This module contains all editor commands sent between JS GUI and Rust.
///
/// It also contains dispatcher for those commands.
use std::string::ToString;
use std::sync::Arc;

use vst_gui::JavascriptCallback;

use crate::parameters::{Pad, Parameters};

pub(crate) fn make_dispatcher(parameters: Arc<Parameters>) -> JavascriptCallback {
    Box::new(move |message: String| -> String {
        let command_str = message.split_whitespace().next().unwrap_or("");
        let command = Command::from(command_str);

        match command {
            Command::SendCode => on_send_code(message, &parameters),
            Command::GetCode => on_get_code(&parameters),
            Command::EvalCode => on_eval_code(message, &parameters),
            Command::EvalSnippetAt => on_eval_snippet_at(message, &parameters),
            Command::SendConsoleOut => on_send_console_out(message, &parameters),
            Command::PostStderr => on_post_stderr(message, &parameters),
            Command::PostStdout => on_post_stdout(message, &parameters),
            Command::GetConsoleOut => on_get_console_out(&parameters),
            Command::SelectPad => on_select_pad(message, &parameters),
            Command::GetSelectedPad => on_get_selected_pad(&parameters),
            Command::SetPadName => on_set_pad_name(message, &parameters),
            Command::GetPadName => on_get_pad_name(message, &parameters),
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

fn on_eval_snippet_at(message: String, parameters: &Parameters) -> String {
    let command_str = Command::EvalSnippetAt.to_string();
    let number = &message[command_str.len() + 1..];
    if let Ok(index) = number.parse::<usize>() {
        parameters.eval_snippet_at(index);
    }
    String::new()
}

fn on_send_console_out(message: String, parameters: &Parameters) -> String {
    let command_str = Command::SendConsoleOut.to_string();
    let out = &message[command_str.len() + 1..];
    parameters.set_console_out(out);
    String::new()
}

fn on_post_stderr(message: String, parameters: &Parameters) -> String {
    let command_str = Command::PostStderr.to_string();
    let out = &message[command_str.len() + 1..];
    parameters.post_stderr(out);
    String::new()
}

fn on_post_stdout(message: String, parameters: &Parameters) -> String {
    let command_str = Command::PostStdout.to_string();
    let out = &message[command_str.len() + 1..];
    parameters.post_stdout(out);
    String::new()
}

fn on_get_console_out(parameters: &Parameters) -> String {
    parameters.console_out()
}

fn on_select_pad(message: String, parameters: &Parameters) -> String {
    let command_str = Command::SelectPad.to_string();
    let out = &message[command_str.len() + 1..];
    if let Ok(pad) = serde_json::from_str::<Pad>(out) {
        parameters.set_selected_pad(pad);
    }
    String::new()
}

fn on_get_selected_pad(parameters: &Parameters) -> String {
    let pad = parameters.selected_pad();
    match serde_json::to_string(&pad) {
        Ok(res) => res,
        Err(e) => format!("{}", e)
    }
}

fn on_set_pad_name(message: String, parameters: &Parameters) -> String {
    let command_str = Command::SetPadName.to_string();
    let pad_json = &message[command_str.len() + 1..];
    if let Ok(pad) = serde_json::from_str::<Pad>(pad_json) {
        parameters.set_pad_name(pad);
    }
    String::new()
}

fn on_get_pad_name(message: String, parameters: &Parameters) -> String {
    let command_str = Command::GetPadName.to_string();
    let number = &message[command_str.len() + 1..];
    if let Ok(number) = number.parse::<usize>() {
        parameters.pad_name_at(number)
    } else {
        String::new()
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    SendCode,
    GetCode,
    EvalCode,
    EvalSnippetAt,
    SendConsoleOut,
    GetConsoleOut,
    PostStderr,
    PostStdout,
    SelectPad,
    GetSelectedPad,
    GetPadName,
    SetPadName,
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
            "EVAL_SNIPPET_AT" => Self::EvalSnippetAt,
            "SEND_CONSOLE_OUT" => Self::SendConsoleOut,
            "GET_CONSOLE_OUT" => Self::GetConsoleOut,
            "POST_STDERR" => Self::PostStderr,
            "POST_STDOUT" => Self::PostStdout,
            "SELECT_PAD" => Self::SelectPad,
            "GET_SELECTED_PAD" => Self::GetSelectedPad,
            "GET_PAD_NAME" => Self::GetPadName,
            "SET_PAD_NAME" => Self::SetPadName,
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
            Self::EvalSnippetAt => "EVAL_SNIPPET_AT".to_string(),
            Self::SendConsoleOut => "SEND_CONSOLE_OUT".to_string(),
            Self::GetConsoleOut => "GET_CONSOLE_OUT".to_string(),
            Self::PostStderr => "POST_STDERR".to_string(),
            Self::PostStdout => "POST_STDOUT".to_string(),
            Self::SelectPad => "SELECT_PAD".to_string(),
            Self::GetSelectedPad => "GET_SELECTED_PAD".to_string(),
            Self::GetPadName => "GET_PAD_NAME".to_string(),
            Self::SetPadName => "SET_PAD_NAME".to_string(),
            Self::Unknown => String::new(),
        }
    }
}
