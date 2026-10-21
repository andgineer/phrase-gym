use egui::{TextEdit, Ui};

use super::intake::{Command, IntakeState};

pub fn show(ui: &mut Ui, state: &mut IntakeState, _draft: &str) {
    ui.separator();
    ui.label("Paste one exact English phrase into Material draft, or drop one plain .txt file, to add it automatically with generated reference speech.");
    ui.label("No title, situation or performer is required for text-only input. Ambiguous meaning and missing preferences fail inline.");
    ui.add_enabled_ui(!state.busy, |ui| {
        if ui.button("Retry connection and retained input").clicked() {
            state.command = Some(Command::Connect { server: state.server.clone() });
        }
        if ui.button("Retry preparation after fixing the problem").clicked() { state.command = Some(Command::RetryPreparation); }
        ui.collapsing("Server connection", |ui| {
            let changed = field(ui, "Local Server URL (optional port override)", &mut state.server, false);
            if changed { state.connected = false; state.status = "Connection settings changed; save the Server URL to reconnect".into(); }
            ui.label("Server and Coach connect directly on this computer. The default port needs no setup.");
            if ui.button("Save connection setup").clicked() {
                state.connected = false;
                state.command = Some(Command::Connect { server: state.server.clone() });
            }
        });
    });
    ui.label(&state.status);
    if let Some(url) = &state.practice {
        ui.hyperlink_to("Open browser practice", url);
    }
}

fn field(ui: &mut Ui, name: &str, value: &mut String, secret: bool) -> bool {
    let label = ui.label(name);
    ui.add(
        TextEdit::singleline(value)
            .password(secret)
            .desired_width(f32::INFINITY),
    )
    .labelled_by(label.id)
    .changed()
}
