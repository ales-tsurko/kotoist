use std::string::ToString;

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
