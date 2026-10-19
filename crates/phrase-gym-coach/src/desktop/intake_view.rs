use egui::{TextEdit, Ui};

use super::intake::{Command, IntakeState};

pub fn show(ui: &mut Ui, state: &mut IntakeState, json: &str) {
    ui.separator();
    ui.label("Supplied lesson preview — explicit Send only");
    ui.add_enabled_ui(!state.busy, |ui| {
        field(ui, "Prepared lesson file", &mut state.lesson_file, false);
        if ui.button("Load prepared lesson").clicked() {
            state.command = Some(Command::Load(state.lesson_file.clone()));
        }
        field(ui, "Reference audio file", &mut state.audio, false);
        let changed = field(ui, "Local Server URL", &mut state.server, false)
            | field(ui, "Material-submit token", &mut state.submit, true)
            | field(ui, "Worker token", &mut state.worker, true);
        if changed {
            state.connected = false;
            state.status = "Connection settings changed; Connect before sending".into();
        }
        ui.label("Tokens are kept in the OS credential store. Leave blank to reuse saved tokens for this Server.");
        if ui.button("Connect").clicked() {
            state.connected = false;
            state.command = Some(Command::Connect {
                server: state.server.clone(),
                submit: std::mem::take(&mut state.submit),
                worker: std::mem::take(&mut state.worker),
            });
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(state.connected && !json.trim().is_empty() && !state.audio.trim().is_empty(), egui::Button::new("Send supplied lesson")).clicked() {
                state.command = Some(Command::Send { json: json.into(), audio: state.audio.clone() });
            }
            if ui.add_enabled(state.connected, egui::Button::new("Retry pending Send")).clicked() {
                state.command = Some(Command::Retry);
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
