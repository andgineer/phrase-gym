#![cfg(feature = "desktop")]

use std::path::PathBuf;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use phrase_gym_coach::adapter::CliAdapter;
use phrase_gym_coach::desktop::appearance;
use phrase_gym_coach::desktop::state::{
    CredentialHealth, FolderHealth, StartupEvent, StatusSnapshot, ThemeChoice, ToolReadiness,
    ViewState,
};
use phrase_gym_coach::desktop::view;
use phrase_gym_coach::platform::paths::CoachPaths;

fn harness(state: ViewState) -> Harness<'static, ViewState> {
    Harness::builder()
        .with_size([560.0, 640.0])
        .with_context_setup(appearance::install)
        .build_ui_state(view::show, state)
}

fn starting() -> ViewState {
    ViewState::new(StatusSnapshot::starting())
}

#[test]
fn startup_shows_a_paused_unconnected_status_through_accessible_names() {
    let mut harness = harness(starting());
    harness.run();
    harness.get_by_label("Phrase Gym Coach");
    harness.get_by_label_contains("Paused");
    harness.get_by_label("Not configured");
    assert_eq!(harness.get_all_by_label("Checking…").count(), 4);
    assert!(harness.ctx.global_style().interaction.selectable_labels);
}

#[test]
fn startup_events_replace_checking_rows_without_waiting() {
    let mut harness = harness(starting());
    harness.run();
    let paths = CoachPaths {
        config: PathBuf::from("/Users/učenik/Library/Application Support/Coach"),
        cache: PathBuf::from("/Users/učenik/Library/Caches/Coach"),
        logs: PathBuf::from("/Users/učenik/Library/Logs/Phrase Gym Coach"),
        state: PathBuf::from("/Users/učenik/Library/Application Support/Coach state"),
    };
    for event in [
        StartupEvent::Tool(
            CliAdapter::Codex,
            ToolReadiness::Found(PathBuf::from("/opt/Tööls Ђ/codex")),
        ),
        StartupEvent::Tool(CliAdapter::Claude, ToolReadiness::Missing),
        StartupEvent::Credentials(CredentialHealth::Denied("user canceled".to_owned())),
        StartupEvent::Folders(FolderHealth::Ready(paths)),
    ] {
        harness.state_mut().snapshot.apply(event);
    }
    harness.run();
    assert!(harness.query_by_label("Checking…").is_none());
    harness.get_by_label("Found at /opt/Tööls Ђ/codex");
    harness.get_by_label("Not found in its documented install locations");
    harness.get_by_label_contains("Access denied (user canceled)");
    harness.get_by_label("/Users/učenik/Library/Logs/Phrase Gym Coach");
}

#[test]
fn keyboard_focus_reaches_the_theme_choice_and_space_selects_it() {
    let mut harness = harness(starting());
    harness.run();
    let mut reached = false;
    for _ in 0..30 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        if harness
            .get_by_role_and_label(Role::RadioButton, "Dark")
            .is_focused()
        {
            reached = true;
            break;
        }
    }
    assert!(reached, "Tab never focused the Dark theme choice");
    harness.key_press(egui::Key::Space);
    harness.run();
    assert_eq!(harness.state().theme, ThemeChoice::Dark);
    assert_eq!(harness.ctx.theme(), egui::Theme::Dark);
}

#[test]
fn the_draft_accepts_german_and_serbian_latin_and_cyrillic_text() {
    let mut harness = harness(starting());
    harness.run();
    let text =
        "Grüße aus Köln, Straße. Ćao, đače! Čuješ li šum žita? Ћао, ђаче! Чујеш ли шум жита?";
    harness.get_by_label("Material draft").click();
    harness.run();
    harness.get_by_label("Material draft").type_text(text);
    harness.run();
    assert_eq!(harness.state().draft, text);
    assert_eq!(
        harness.get_by_label("Material draft").value().as_deref(),
        Some(text)
    );
}

#[test]
fn text_size_controls_scale_the_interface_within_bounds() {
    let mut harness = harness(starting());
    harness.run();
    harness.get_by_label("Larger text").click();
    harness.run();
    assert!((harness.ctx.zoom_factor() - 1.25).abs() < f32::EPSILON);
    harness.get_by_label_contains("Text size 125%");
    for _ in 0..6 {
        harness.get_by_label("Larger text").click();
        harness.run();
    }
    assert!((harness.ctx.zoom_factor() - appearance::MAX_TEXT_SCALE).abs() < f32::EPSILON);
    harness.get_by_label("Reset text size").click();
    harness.run();
    assert!((harness.ctx.zoom_factor() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn every_control_stays_named_at_the_minimum_window_size_on_high_dpi_in_both_themes() {
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        let mut harness = Harness::builder()
            .with_size([380.0, 320.0])
            .with_pixels_per_point(2.0)
            .with_theme(theme)
            .with_context_setup(appearance::install)
            .build_ui_state(view::show, starting());
        harness.run();
        harness.check_accessibility();
        harness.get_by_role_and_label(Role::RadioButton, "System");
        harness.get_by_label("Material draft");
    }
}
