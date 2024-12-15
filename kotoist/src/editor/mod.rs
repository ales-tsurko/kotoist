use std::sync::{Arc, Mutex};

use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use nih_plug::editor::Editor;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, containers::ScrollArea},
};

use self::piano_roll::PianoRoll;
use crate::parameters::{InterpreterMessage, Parameters, Snippet};
use crate::pipe::{Message as PipeMessage, PipeOut};

mod piano_roll;

pub(crate) const WINDOW_SIZE: (u32, u32) = (700, 734);

pub(crate) fn create_editor(
    params: Arc<Parameters>,
    pipe_out: Arc<Mutex<PipeOut>>,
) -> Option<Box<dyn Editor>> {
    let piano_roll = PianoRoll::default();
    create_egui_editor(
        params.editor_state.clone(),
        GuiState::default(),
        |_, _| {},
        move |egui_ctx, _setter, state| {
            egui::TopBottomPanel::top("tabbar")
                .show_separator_line(false)
                .exact_height(42.0)
                .show(egui_ctx, |ui| {
                    tab_bar(ui, params.clone());
                });

            egui::TopBottomPanel::bottom("console")
                .exact_height(200.0)
                .show_separator_line(false)
                .show(egui_ctx, |ui| bottom_panel(ui, state, &params, &pipe_out));

            // per egui docs the CentralPanel should always go last
            let bg = egui::Visuals::default().faint_bg_color;
            egui::CentralPanel::default()
                .frame(egui::Frame::default().inner_margin(7.0).fill(bg))
                .show(egui_ctx, |ui| {
                    let rect = ui.available_rect_before_wrap();

                    ui.allocate_ui_at_rect(rect, |ui| {
                        piano_roll.draw(ui);
                    });

                    ui.allocate_ui_at_rect(rect, |ui| {
                        text_editor(egui_ctx, ui, state, &params);
                    });
                });
        },
    )
}

fn tab_bar(ui: &mut egui::Ui, params: Arc<Parameters>) {
    ScrollArea::horizontal().show(ui, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let selected_snippet = params.selected_snippet_index();
            if let Ok(snippets) = params.snippets.try_read() {
                for (index, snippet) in snippets.iter().enumerate() {
                    let fallback_index = index
                        .min(snippets.len().saturating_sub(2))
                        .min(selected_snippet);
                    show_tab(
                        ui,
                        snippet,
                        index,
                        fallback_index,
                        index == selected_snippet,
                        params.clone(),
                    );
                }
            }

            if ui.button("+").clicked() {
                params.send_interpreter_msg(InterpreterMessage::AddSnippet);
            }
        });
    });
}

fn show_tab(
    ui: &mut egui::Ui,
    snippet: &Snippet,
    index: usize,
    fallback_index: usize,
    selected: bool,
    params: Arc<Parameters>,
) {
    if ui
        .selectable_label(selected, egui::RichText::new(&snippet.name).size(22.0))
        .clicked()
    {
        params.set_selected_snippet_index(index);
    }

    if ui.selectable_label(false, "x").clicked() {
        params.send_interpreter_msg(InterpreterMessage::RemoveSnippet(index));
        params.set_selected_snippet_index(fallback_index);
    }
}

fn text_editor(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    state: &mut GuiState,
    params: &Arc<Parameters>,
) {
    ScrollArea::both().stick_to_bottom(true).show(ui, |ui| {
        // we need to sync the state
        {
            let index = params.selected_snippet_index();
            if let Some(snippet) = params
                .snippets
                .try_read()
                .ok()
                .as_ref()
                .and_then(|s| s.get(index))
            {
                if state.text_buffer != snippet.code {
                    state.text_buffer = snippet.code.clone();
                }
            }
        }

        let output = CodeEditor::default()
            .id_source("code editor")
            .with_rows(31)
            .with_fontsize(13.0)
            .vscroll(false)
            .with_theme(ColorTheme::AYU_MIRAGE)
            .with_syntax(koto_syntax())
            .with_numlines(true)
            .show(ui, &mut state.text_buffer);

        if output.response.changed() {
            let index = params.selected_snippet_index();
            params.send_interpreter_msg(InterpreterMessage::SetSnippetCode(
                index,
                state.text_buffer.clone(),
            ));
        }

        let selected_text = output
            .cursor_range
            .map(|text_cursor_range| {
                use egui::TextBuffer as _;
                let selected_chars = text_cursor_range.as_sorted_char_range();
                state.text_buffer.char_range(selected_chars)
            })
            .unwrap_or_default();

        if ui.interact_bg(egui::Sense::click()).clicked() {
            output.response.request_focus();
        }

        ctx.input_mut(|i| {
            // Cmd-Enter - eval all/selection
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Enter,
            )) {
                let code = if selected_text.is_empty() {
                    &state.text_buffer
                } else {
                    selected_text
                };

                params.send_interpreter_msg(InterpreterMessage::EvalCode(code.to_owned()));
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
            ui.label("Console");
            ui.add_space(6.0);
        });
        ui.add_space(ui.available_width() - 50.0);
        if ui.button("🚀 Run").clicked() {
            params.send_interpreter_msg(InterpreterMessage::EvalCode(state.text_buffer.clone()));
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

#[derive(Default)]
struct GuiState {
    console: Vec<PipeMessage>,
    // to prevent locks, we clone the whole snippet code, when the selected snippet is changed. then
    // we update the snippet in params only when this value is update
    text_buffer: String,
}
