#![cfg(feature = "desktop")]

use std::time::Duration;

use egui::{Pos2, RawInput, Rect, ViewportId, vec2};
use phrase_gym_coach::desktop::appearance;
use phrase_gym_coach::desktop::state::{StatusSnapshot, ViewState};
use phrase_gym_coach::desktop::view;

// Repaint delay egui requests after each of several idle frames of a focused window.
fn idle_delays(settings: bool, blink: bool) -> Vec<Duration> {
    let context = egui::Context::default();
    appearance::install(&context);
    if blink {
        context.all_styles_mut(|style| style.visuals.text_cursor.blink = true);
    }
    let mut state = ViewState::new(StatusSnapshot::starting());
    state.settings_open = settings;
    state.intake.summary = Some("1 lessons · 5 phrases · 5 ready · 0 awaiting audio".into());
    (0..12)
        .map(|frame| {
            let input = RawInput {
                focused: true,
                time: Some(f64::from(frame) * 0.4),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(560.0, 560.0))),
                ..RawInput::default()
            };
            let mut output = context.run_ui(input, |ui| {
                if settings {
                    view::settings(ui, &mut state);
                } else {
                    view::show(ui, &mut state);
                }
            });
            output.textures_delta.clear();
            output.viewport_output[&ViewportId::ROOT].repaint_delay
        })
        .collect()
}

#[test]
fn an_idle_focused_main_window_and_settings_request_no_further_frames() {
    for settings in [false, true] {
        let delays = idle_delays(settings, false);
        assert!(
            delays[4..].iter().all(|delay| *delay == Duration::MAX),
            "settings={settings}: {delays:?}"
        );
    }
}

#[test]
fn a_blinking_caret_in_the_focused_material_box_would_wake_the_idle_window() {
    let delays = idle_delays(false, true);
    assert!(
        delays[4..]
            .iter()
            .all(|delay| *delay < Duration::from_secs(1)),
        "{delays:?}"
    );
}
