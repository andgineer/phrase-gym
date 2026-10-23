#![cfg(feature = "desktop")]

use egui::accesskit::Role;
use egui::{FontId, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use phrase_gym_coach::adapter::CliAdapter;
use phrase_gym_coach::desktop::appearance;
use phrase_gym_coach::desktop::intake::Command;
use phrase_gym_coach::desktop::state::{StatusSnapshot, ViewState};
use phrase_gym_coach::desktop::tool_choice::ChoiceEvent;
use phrase_gym_coach::desktop::view::{self, SEND_HINT};
use phrase_gym_coach::transfer::input_problem::InputProblem;

fn harness(state: ViewState) -> Harness<'static, ViewState> {
    Harness::builder()
        .with_size([560.0, 900.0])
        .with_context_setup(appearance::install)
        .build_ui_state(
            |ui, state| {
                if state.settings_open {
                    view::settings(ui, state);
                } else {
                    view::show(ui, state);
                }
            },
            state,
        )
}

fn typed(text: &str) -> Harness<'static, ViewState> {
    let mut harness = harness(ViewState::new(StatusSnapshot::starting()));
    harness.run();
    harness.get_by_label("English material").click();
    harness.run();
    harness.get_by_label("English material").type_text(text);
    harness.run();
    harness
}

#[test]
fn typed_text_waits_with_a_visible_hint_until_the_send_shortcut() {
    let mut harness = typed("impediment\nCould you repeat that, please?");
    assert!(harness.state().intake.command.is_none());
    harness.get_by_label(SEND_HINT);
    assert!(harness.query_by_role(Role::Button).is_none());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().intake.command,
        Some(Command::Text(
            "impediment\nCould you repeat that, please?".into()
        ))
    );
    assert_eq!(
        harness.state().draft,
        "impediment\nCould you repeat that, please?"
    );
    harness.get_by_label(SEND_HINT);
}

#[test]
fn plain_return_only_starts_a_new_line() {
    let mut harness = typed("impediment");
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.state().intake.command.is_none());
    assert_eq!(harness.state().draft, "impediment\n");
}

#[test]
fn the_send_shortcut_explains_why_nothing_was_sent_while_busy_or_empty() {
    let mut harness = typed("impediment");
    harness.state_mut().intake.busy = true;
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.run_steps(2);
    assert!(harness.state().intake.command.is_none());
    assert_eq!(harness.state().intake.status, InputProblem::Busy.message());
    harness.state_mut().intake.busy = false;
    harness.run();
    harness.get_by_label(InputProblem::Busy.message());
    let mut empty = harness_with_draft("  \n ");
    empty.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    empty.run();
    assert!(empty.state().intake.command.is_none());
    empty.get_by_label(InputProblem::Empty.message());
}

fn harness_with_draft(draft: &str) -> Harness<'static, ViewState> {
    let mut state = ViewState::new(StatusSnapshot::starting());
    state.draft = draft.into();
    let mut harness = harness(state);
    harness.run();
    harness
}

#[test]
fn the_send_hint_renders_with_the_bundled_fonts() {
    let harness = harness(ViewState::new(StatusSnapshot::starting()));
    assert!(
        harness
            .ctx
            .fonts_mut(|fonts| fonts.has_glyphs(&FontId::proportional(12.0), SEND_HINT))
    );
}

#[test]
fn settings_switch_the_preparation_tool_with_one_control() {
    let mut state = ViewState::new(StatusSnapshot::starting());
    state.settings_open = true;
    let mut harness = harness(state);
    harness.run();
    harness
        .get_by_role_and_label(Role::RadioButton, "Claude CLI")
        .click();
    harness.run();
    assert_eq!(harness.state().tool.command, None);
    harness
        .state_mut()
        .tool
        .apply(ChoiceEvent::Loaded(CliAdapter::Codex));
    harness.run();
    harness
        .get_by_role_and_label(Role::RadioButton, "Claude CLI")
        .click();
    harness.run();
    assert_eq!(harness.state().tool.command, Some(CliAdapter::Claude));
    assert_eq!(harness.state().tool.selected, Some(CliAdapter::Claude));
    harness.state_mut().tool.command = None;
    harness
        .state_mut()
        .tool
        .apply(ChoiceEvent::Saved(CliAdapter::Claude));
    harness.run();
    harness.get_by_label("Claude CLI prepares your next material.");
    harness
        .get_by_role_and_label(Role::RadioButton, "Claude CLI")
        .click();
    harness.run();
    assert_eq!(harness.state().tool.command, None);
}
