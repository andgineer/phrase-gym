use chrono::TimeDelta;
use serde_json::{Value, json};

use crate::support::TestServer;
use crate::support::content::package;
use crate::support::practice::{
    advance, apply, moment, rate, ready, schedule_version, session, started,
};

async fn practising() -> (TestServer, Value) {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    (server, state)
}

async fn run_cards(server: &TestServer, mut state: Value, spacing: TimeDelta) -> Value {
    for rating in ["not_yet", "recalled", "recalled"] {
        state = rate(server, &state, rating).await["session"].clone();
        advance(server, spacing);
    }
    state
}

#[tokio::test]
async fn quick_other_cards_complete_the_session_without_waiting_for_a_repair() {
    let (server, state) = practising().await;
    let state = run_cards(&server, state, TimeDelta::zero()).await;
    assert_eq!(state["status"], "completed");
    assert_eq!(state["queue"].as_array().unwrap().len(), 3);
    assert_eq!(state["attempts"][0]["rating"], "not_yet");
}

#[tokio::test]
async fn a_spaced_repair_keeps_the_original_result_and_updates_the_schedule_once() {
    let (server, state) = practising().await;
    let state = run_cards(&server, state, TimeDelta::minutes(3)).await;
    assert_eq!(
        (&state["status"], &state["cursor"]),
        (&json!("active"), &json!(3))
    );
    assert_eq!(state["queue"][3]["repair"], true);
    let (_, presented) = apply(&server, &state, "present", &json!({})).await;
    let recalled = json!({"rating": "recalled"});
    let (_, result) = apply(&server, &presented["session"], "rate", &recalled).await;
    assert_eq!(result["receipts"][0]["eligibility_reason"], "repair");
    assert_eq!(result["receipts"][0]["schedule_changed"], false);
    assert_eq!(result["session"]["attempts"][0]["rating"], "not_yet");
    assert_eq!(
        schedule_version(&server, &state["attempts"][0]["target_id"]),
        Some(1)
    );
}

#[tokio::test]
async fn supported_responses_are_repaired_but_a_target_never_gets_more_than_two_extras() {
    let (server, mut state) = practising().await;
    for _ in 0..3 {
        let (_, presented) = apply(&server, &state, "present", &json!({})).await;
        let (_, revealed) = apply(&server, &presented["session"], "explain", &json!({})).await;
        let effort = json!({"rating": "with_effort"});
        state = apply(&server, &revealed["session"], "rate", &effort)
            .await
            .1["session"]
            .clone();
        advance(&server, TimeDelta::minutes(3));
    }
    for _ in 0..6 {
        if state["status"] != "active" {
            break;
        }
        state = rate(&server, &state, "not_yet").await["session"].clone();
        advance(&server, TimeDelta::minutes(3));
    }
    let queue = state["queue"].as_array().unwrap();
    for index in 0..3 {
        let extras = queue
            .iter()
            .filter(|step| step["repair"] == true && step["target_index"] == index)
            .count();
        assert_eq!(extras, 2, "target {index}");
    }
    assert_eq!(state["status"], "completed");
    let saved = session(&server.router, &state["id"]).await;
    assert!(
        saved["attempts"].as_array().unwrap()[..3]
            .iter()
            .all(|attempt| attempt["original"]["supported"] == true)
    );
}
