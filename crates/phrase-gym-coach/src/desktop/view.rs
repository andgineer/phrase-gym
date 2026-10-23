use egui::{Grid, Key, Label, Modifiers, RichText, ScrollArea, TextEdit, Ui};

use super::appearance::{self, MAX_TEXT_SCALE, MIN_TEXT_SCALE};
use super::intake::Command;
use super::state::{StatusSnapshot, ThemeChoice, ToolChoice, ViewState};
use super::text;
use crate::adapter::CliAdapter;
use crate::transfer::input_problem::InputProblem;

pub const TITLE: &str = "Phrase Gym Coach";
#[cfg(target_os = "macos")]
pub const SEND_HINT: &str = "Paste, drop a text file, or type and press Command-Return";
#[cfg(not(target_os = "macos"))]
pub const SEND_HINT: &str = "Paste, drop a text file, or type and press Ctrl+Enter";
const TEXT_STEP: f32 = 0.25;
const THEMES: [(ThemeChoice, &str); 3] = [
    (ThemeChoice::System, "System"),
    (ThemeChoice::Light, "Light"),
    (ThemeChoice::Dark, "Dark"),
];

// Renders the snapshot and edits only view-local state; it performs no I/O, so a slow disk,
// tool discovery or child process can never stall a frame.
pub fn show(ui: &mut Ui, state: &mut ViewState) {
    state.text_scale = ui.ctx().zoom_factor();
    egui::Frame::central_panel(ui.style()).show(ui, |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.add_space(24.0);
            ui.heading("Your next working set");
            ui.add_space(24.0);
            if draft(ui, &mut state.draft, !state.settings_open) {
                state.intake.status = String::new();
                if state.intake.busy {
                    state.intake.status = InputProblem::Busy.into();
                } else if state.draft.trim().is_empty() {
                    state.intake.status = InputProblem::Empty.into();
                } else {
                    state.intake.command = Some(Command::Text(state.draft.clone()));
                }
            }
            ui.add_space(16.0);
            if state.intake.busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Preparing your material…");
                });
            } else if let Some(message) =
                text::intake_status(&state.intake.status, state.tool.selected)
            {
                ui.label(message);
            }
            if let Some(summary) = &state.intake.summary {
                ui.label(summary);
            }
            if let Some(url) = &state.intake.practice {
                ui.hyperlink_to("Open browser practice", url);
            }
        });
    });
}

fn status(ui: &mut Ui, snapshot: &StatusSnapshot) {
    ui.label(RichText::new("Status").strong());
    Grid::new("status")
        .num_columns(2)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            row(
                ui,
                "Processing",
                text::processing(snapshot.processing_enabled),
            );
            row(ui, "Server", text::server(snapshot.server_configured));
            for (adapter, readiness) in &snapshot.tools {
                row(ui, adapter.display_name(), &text::tool(readiness));
            }
            for (name, value) in text::folders(&snapshot.folders) {
                row(ui, name, &value);
            }
        });
}

fn row(ui: &mut Ui, name: &str, value: &str) {
    ui.label(name);
    ui.add(Label::new(value).selectable(true).wrap());
    ui.end_row();
}

fn appearance_controls(ui: &mut Ui, state: &mut ViewState) {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.label("Theme");
        for (choice, name) in THEMES {
            changed |= ui.radio_value(&mut state.theme, choice, name).changed();
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("Text size {:.0}%", state.text_scale * 100.0));
        let smaller = state.text_scale > MIN_TEXT_SCALE;
        if ui
            .add_enabled(smaller, egui::Button::new("Smaller text"))
            .clicked()
        {
            state.text_scale -= TEXT_STEP;
            changed = true;
        }
        let larger = state.text_scale < MAX_TEXT_SCALE;
        if ui
            .add_enabled(larger, egui::Button::new("Larger text"))
            .clicked()
        {
            state.text_scale += TEXT_STEP;
            changed = true;
        }
        if ui.button("Reset text size").clicked() {
            state.text_scale = 1.0;
            changed = true;
        }
    });
    if changed {
        appearance::apply(ui.ctx(), state.theme, state.text_scale);
    }
}

// A paste, or the send shortcut for typed text, is the deliberate gesture; typing alone is not.
fn draft(ui: &mut Ui, draft: &mut String, focus: bool) -> bool {
    // Consumed before the text box runs, so the shortcut never also inserts a line break.
    let send = ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::Enter));
    let pasted = ui.ctx().input(|input| {
        input
            .events
            .iter()
            .any(|event| matches!(event, egui::Event::Paste(_)))
    });
    let label = ui.label(RichText::new("English material").strong());
    let response = ui
        .add(
            TextEdit::multiline(draft)
                .hint_text("Drop or paste words and phrases")
                .desired_rows(10)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(label.id);
    ui.label(RichText::new(SEND_HINT).small());
    if focus && ui.memory(|memory| memory.focused().is_none()) {
        response.request_focus();
    }
    (response.changed() && pasted) || send
}

fn tool_controls(ui: &mut Ui, choice: &mut ToolChoice) {
    ui.label(RichText::new("Preparation tool").strong());
    ui.horizontal_wrapped(|ui| {
        for adapter in CliAdapter::ALL {
            let selected = choice.selected == Some(adapter);
            let enabled = choice.selected.is_some();
            if ui
                .add_enabled(
                    enabled,
                    egui::RadioButton::new(selected, adapter.display_name()),
                )
                .clicked()
                && !selected
            {
                choice.selected = Some(adapter);
                choice.command = Some(adapter);
            }
        }
    });
    if !choice.status.is_empty() {
        ui.label(&choice.status);
    }
}

pub fn settings(ui: &mut Ui, state: &mut ViewState) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Settings");
        tool_controls(ui, &mut state.tool);
        ui.separator();
        appearance_controls(ui, state);
        ui.separator();
        super::intake_view::show(ui, &mut state.intake, &state.draft);
        ui.separator();
        status(ui, &state.snapshot);
    });
}
