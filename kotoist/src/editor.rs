use std::sync::{atomic::Ordering, Arc, Mutex};

use nih_plug::editor::Editor;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, containers::ScrollArea},
};

use crate::parameters::Parameters;
use crate::pipe::{Message as PipeMessage, PipeOut};

pub(crate) const WINDOW_SIZE: (u32, u32) = (800, 600);

pub(crate) fn create_editor(
    params: Arc<Parameters>,
    pipe_out: Arc<Mutex<PipeOut>>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        GuiState::default(),
        |_, _| {},
        move |egui_ctx, _setter, state| {
            egui::SidePanel::left("pad-selector")
                .min_width(150.0)
                .show(egui_ctx, |ui| {
                    side_panel(ui, params.clone());
                });

            egui::TopBottomPanel::bottom("console")
                .exact_height(150.0)
                .show(egui_ctx, |ui| bottom_panel(ui, state, &pipe_out));

            // per egui docs the CentralPanel should always go last
            let bg = egui::Visuals::default().faint_bg_color;
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(bg))
                .show(egui_ctx, |ui| {
                    text_editor(egui_ctx, ui, &params);
                });
        },
    )
}

fn side_panel(ui: &mut egui::Ui, params: Arc<Parameters>) {
    ui.add_space(6.0);
    ui.label("Snippets");
    ui.add_space(6.0);
}

fn text_editor(ctx: &egui::Context, ui: &mut egui::Ui, params: &Arc<Parameters>) {
    ScrollArea::both().show(ui, |ui| {
        let index = params.selected_snippet_index();
        let mut snippets = params.snippets.write().unwrap();
        let output = egui::TextEdit::multiline(&mut snippets[index].code)
            .code_editor()
            .min_size(ui.available_size())
            .frame(false)
            .desired_width(f32::INFINITY)
            .show(ui);

        let selected_text = output
            .cursor_range
            .map(|text_cursor_range| {
                use egui::TextBuffer as _;
                let selected_chars = text_cursor_range.as_sorted_char_range();
                snippets[index].code.char_range(selected_chars)
            })
            .unwrap_or_default();

        ctx.input_mut(|i| {
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Enter,
            )) {
                let code = if selected_text.is_empty() {
                    &snippets[index].code
                } else {
                    selected_text
                };

                params.eval_code(code);
            }
        });
    });
}

fn bottom_panel(ui: &mut egui::Ui, state: &mut GuiState, pipe_out: &Arc<Mutex<PipeOut>>) {
    ui.add_space(6.0);
    ui.label("Console output");
    ui.add_space(6.0);
    ScrollArea::both().stick_to_bottom(true).show(ui, |ui| {
        let pipe = pipe_out.lock().unwrap();
        if let Ok(out) = pipe.receiver.try_recv() {
            state.console.push(out);
        }

        for out in &state.console {
            match out {
                PipeMessage::Normal(out) => {
                    let out = egui::RichText::new(out).monospace();
                    let _ = ui.label(out);
                }
                PipeMessage::Error(out) => {
                    let color = ui.visuals().error_fg_color;
                    let out = egui::RichText::new(out).monospace().color(color);
                    let _ = ui.label(out);
                }
            }
        }

        ui.allocate_space(ui.available_size());
    });
}

#[derive(Debug, Default)]
struct GuiState {
    console: Vec<PipeMessage>,
}
