use egui::{TextEdit, Ui};

use super::intake::{Command, IntakeState};

pub fn show(ui: &mut Ui, state: &mut IntakeState, _draft: &str) {
    ui.separator();
    ui.label("One English phrase and its original WAV reference (up to 1 MB)");
    ui.add_enabled_ui(!state.busy, |ui| {
        if state.prepared.is_none() {
            field(ui, "Lesson title (up to 200 bytes)", &mut state.title, false);
            field(ui, "Situation or meaning (up to 800 bytes)", &mut state.situation, false);
            field(ui, "Reference performer (up to 200 bytes)", &mut state.performer, false);
            field(ui, "Audio locale", &mut state.locale, false);
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("Choose prepared lesson…").clicked() { state.command = Some(Command::ChooseLesson); }
            if ui.button("Load supplied demo text").clicked() { state.command = Some(Command::Demo); }
        });
        if let Some(preview) = &state.preview {
            ui.heading(&preview.title);
            ui.label(&preview.phrase);
            for situation in &preview.situations { ui.label(format!("Situation: {situation}")); }
            if let Some(help) = &preview.help { ui.label(format!("Meaning: {help}")); }
            ui.label(&preview.audio);
            ui.label("Text validated locally. Choose its matching original audio to start automatic publication.");
            if ui.button("Use raw English input instead").clicked() { state.prepared = None; state.preview = None; }
        }
        if ui.button("Choose original audio and add lesson…").clicked() { state.command = Some(Command::ChooseAudio); }
        ui.label("Choosing or dropping audio starts delivery automatically. Opening a picker alone sends nothing.");
        if !state.audio.is_empty() { ui.label(format!("Selected reference: {}", state.audio.rsplit(['/', '\\']).next().unwrap_or("audio"))); }
        if !state.audio.is_empty() && ui.button("Preview selected original audio").clicked() { state.command = Some(Command::PreviewAudio(state.audio.clone())); }
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
