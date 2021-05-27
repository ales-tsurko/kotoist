use log::info;
use vst_gui::PluginGui;

const HTML: &'static str = include_str!("../gui/build/index.html");
const EDITOR_SIZE: (i32, i32) = (640, 480);

pub(crate) struct KotoistEditor {
    gui: PluginGui,
}

impl KotoistEditor {
    pub(crate) fn into_handler(self) -> Box<PluginGui> {
        Box::new(self.gui)
    }
}

impl Default for KotoistEditor {
    fn default() -> Self {
        Self {
            gui: vst_gui::new_plugin_gui(
                String::from(HTML),
                Box::new(Dispatcher::dispatch),
                Some(EDITOR_SIZE),
            ),
        }
    }
}

struct Dispatcher;

impl Dispatcher {
    fn dispatch(message: String) -> String {
        let command_str = message.split_whitespace().next().unwrap_or("");
        let command = Command::from(command_str);

        match command {
            Command::SendCode => Self::on_send_code(message),
            Command::Unknown => String::new(),
        }
    }

    fn on_send_code(message: String) -> String {
        let command_str: String = Command::SendCode.into();
        info!("{}", &message[command_str.len() + 1..]);
        "This is sent from Rust\nAnd this is another line from Rust for testing.".to_string()
    }
}

enum Command {
    SendCode,
    Unknown,
}

impl From<&str> for Command {
    fn from(message_str: &str) -> Self {
        match message_str {
            "SEND_CODE" => Self::SendCode,
            _ => Self::Unknown,
        }
    }
}

impl Into<String> for Command {
    fn into(self) -> String {
        match self {
            Self::SendCode => "SEND_CODE".to_string(),
            Self::Unknown => String::new(),
        }
    }
}
