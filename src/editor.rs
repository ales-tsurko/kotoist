pub(crate) mod command;

use std::sync::{Arc, RwLock};

use vst::editor::{Editor, KeyCode, KnobMode};
use vst_gui::PluginGui;

use self::command::make_dispatcher;
use crate::parameters::Parameters;

const HTML: &'static str = include_str!("../gui/build/index.html");
const EDITOR_SIZE: (i32, i32) = (650, 480);

pub(crate) struct KotoistEditor {
    gui: Arc<RwLock<PluginGui>>,
    parameters: Arc<Parameters>,
}

impl KotoistEditor {
    pub(crate) fn new(parameters: Arc<Parameters>) -> Self {
        let gui = Arc::new(RwLock::new(vst_gui::new_plugin_gui(
            String::from(HTML),
            make_dispatcher(Arc::clone(&parameters)),
            Some(EDITOR_SIZE),
        )));

        Self { gui, parameters }
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
        let mut gui = self.gui.write().unwrap();

        gui.idle();

        // process events
        if let Some(event) = self.parameters.event_queue.write().unwrap().pop_front() {
            let value = event.value;
            let command = event.command;
            gui.execute(&command.to_js_event(&value)).unwrap();
        }
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
