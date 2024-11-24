use std::sync::{Arc, Mutex};

use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
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
                .exact_width(100.0)
                .show(egui_ctx, |ui| {
                    side_panel(ui, params.clone());
                });

            egui::TopBottomPanel::bottom("console")
                .exact_height(150.0)
                .show(egui_ctx, |ui| bottom_panel(ui, state, &params, &pipe_out));

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
    {
        let selection = params.selected_snippet_index();
        let selection = &params.snippets.read().unwrap()[selection];
        ui.add_space(6.0);
        ui.label(format!("Snippets | {}", selection.piano_key.name));
        ui.add_space(6.0);
    }

    ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
        let snippets = params.snippets.read().unwrap();
        for (index, snippet) in snippets.iter().enumerate().rev() {
            let color = if !snippet.code.is_empty() {
                egui::Color32::LIGHT_RED
            } else if snippet.piano_key.is_black {
                egui::Color32::BLACK
            } else {
                egui::Color32::WHITE
            };
            if ui
                .add_sized(
                    [ui.available_width(), 10.0],
                    egui::Button::new(&snippet.piano_key.name).fill(color),
                )
                .clicked()
            {
                params.set_selected_snippet_index(index);
            }
        }
    });
}

fn text_editor(ctx: &egui::Context, ui: &mut egui::Ui, params: &Arc<Parameters>) {
    ScrollArea::both().stick_to_bottom(true).show(ui, |ui| {
        let index = params.selected_snippet_index();
        let mut snippets = params.snippets.write().unwrap();
        let output = CodeEditor::default()
            .id_source("code editor")
            .with_rows(1)
            .with_fontsize(13.0)
            .vscroll(false)
            .with_theme(ColorTheme::AYU_MIRAGE)
            .with_syntax(koto_syntax())
            .with_numlines(true)
            .show(ui, &mut snippets[index].code);

        let selected_text = output
            .cursor_range
            .map(|text_cursor_range| {
                use egui::TextBuffer as _;
                let selected_chars = text_cursor_range.as_sorted_char_range();
                snippets[index].code.char_range(selected_chars)
            })
            .unwrap_or_default();

        if ui.allocate_response(ui.available_size(), egui::Sense::click()).clicked() {
            output.response.request_focus();
        }

        ctx.input_mut(|i| {
            // Cmd-Enter - eval all/selection
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

fn bottom_panel(
    ui: &mut egui::Ui,
    state: &mut GuiState,
    params: &Arc<Parameters>,
    pipe_out: &Arc<Mutex<PipeOut>>,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.label("Console output");
            ui.add_space(6.0);
        });
        ui.add_space(ui.available_width() - 50.0);
        if ui.button("🚀 Run").clicked() {
            let index = params.selected_snippet_index();
            params.eval_snippet_at(index);
        }
    });
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

fn koto_syntax() -> Syntax {
    Syntax::new("koto")
        .with_case_sensitive(true)
        .with_comment("#")
        .with_comment_multiline(["#-", "-#"])
        .with_hyperlinks(["http"])
        .with_keywords([
            "as",
            "assert",
            "assert_eq",
            "assert_ne",
            "assert_near",
            "break",
            "catch",
            "continue",
            "debug",
            "else",
            "export",
            "finally",
            "for",
            "from",
            "if",
            "import",
            "in",
            "loop",
            "match",
            "return",
            "switch",
            "then",
            "throw",
            "try",
            "until",
            "while",
            "yield",
        ])
        .with_special(["false", "null", "self", "true"])
}

#[derive(Debug, Default)]
struct GuiState {
    console: Vec<PipeMessage>,
}
