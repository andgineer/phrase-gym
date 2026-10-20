use egui::{Grid, Label, RichText, ScrollArea, TextEdit, Ui};

use super::appearance::{self, MAX_TEXT_SCALE, MIN_TEXT_SCALE};
use super::state::{StatusSnapshot, ThemeChoice, ViewState};
use super::text;

pub const TITLE: &str = "Phrase Gym Coach";
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
            ui.heading(TITLE);
            ui.add_space(4.0);
            status(ui, &state.snapshot);
            ui.separator();
            appearance_controls(ui, state);
            ui.separator();
            if draft(ui, &mut state.draft) && !state.intake.busy {
                state.intake.command = Some(super::intake::Command::Text(state.draft.clone()));
            }
            super::intake_view::show(ui, &mut state.intake, &state.draft);
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

fn draft(ui: &mut Ui, draft: &mut String) -> bool {
    let pasted = ui.ctx().input(|input| {
        input
            .events
            .iter()
            .any(|event| matches!(event, egui::Event::Paste(_)))
    });
    let label = ui.label(RichText::new("Material draft").strong());
    let response = ui
        .add(
            TextEdit::multiline(draft)
                .hint_text("Paste one exact English phrase (up to 200 bytes).")
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(label.id);
    response.changed() && pasted
}
