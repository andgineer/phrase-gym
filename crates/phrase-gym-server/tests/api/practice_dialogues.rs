use axum::http::StatusCode;
use chrono::TimeDelta;
use phrase_gym_core::clock::Clock;
use phrase_gym_core::timestamps;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::TestServer;
use crate::support::content::{get, package, publish};
use crate::support::practice::{
    apply, count, current, event, exposure, moment, ready, session, start, started, submit,
};

fn server() -> TestServer {
    TestServer::start(moment("2026-10-12T12:00:00+00:00"))
}

// Starts the published dialogue, explicitly replacing any open session.
async fn dialogue_session(server: &TestServer) -> Value {
    let dialogue = get(&server.router, "/api/v1/dialogues?language=en")
        .await
        .json()["items"][0]
        .clone();
    let takeover = current(&server.router).await["id"].clone();
    started(
        &server.router,
        &json!({"dialogue_id": dialogue["id"], "takeover_session_id": takeover}),
    )
    .await
}

fn readiness(body: &Value) -> (i64, i64) {
    (
        body["new_count"].as_i64().unwrap(),
        body["preparing_count"].as_i64().unwrap(),
    )
}

#[tokio::test]
async fn dialogue_script_detours_and_repeated_targets_share_one_schedule() {
    let server = server();
    let mut content = package();
    let catch_up = content["targets"][1].clone();
    content["dialogues"][0]["turns"]
        .as_array_mut()
        .unwrap()
        .extend([
            json!({"speaker": "Partner", "text": "Good idea. Shall we meet again next week?"}),
            json!({"speaker": "Learner", "text": catch_up["text"], "target_source_id": catch_up["source_id"]}),
            json!({"speaker": "Partner", "text": "See you then."}),
        ]);
    ready(&server.router, &mut content).await;
    let state = dialogue_session(&server).await;
    let turns: Vec<_> = state["queue"]
        .as_array()
        .unwrap()
        .iter()
        .map(|step| step["dialogue_turn"].clone())
        .collect();
    assert_eq!(turns, [json!(1), json!(3)]);
    assert_eq!(state["manifest"]["targets"].as_array().unwrap().len(), 1);
    let (_, result) = apply(&server, &state, "dialogue_exposure", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "dialogue_begin", &json!({})).await;
    assert_eq!(result["session"]["queue"][0]["introduced"], true);
    let (_, result) = apply(&server, &result["session"], "present", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "detour", &json!({})).await;
    assert_eq!(result["session"]["queue"][0]["dialogue_turn"], Value::Null);
    assert_eq!(result["session"]["queue"][1]["dialogue_turn"], 3);
    let recalled = json!({"rating": "recalled"});
    let (_, result) = apply(&server, &result["session"], "rate", &recalled).await;
    assert_eq!(
        result["receipts"][0]["eligibility_reason"],
        "supported_learning_check"
    );
    let (_, result) = apply(&server, &result["session"], "present", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "rate", &recalled).await;
    assert_eq!(result["receipts"][0]["schedule_changed"], false);
    assert_eq!(result["session"]["status"], "completed");
    assert_eq!(
        result["session"]["manifest"]["dialogue"]["content"]["turns"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    let (schedules, memory): (i64, Option<String>) =
        rusqlite::Connection::open(&server.database_path)
            .unwrap()
            .query_row(
                "SELECT count(*), max(memory_state) FROM target_schedule",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
    assert_eq!((schedules, memory), (1, None));
}

#[tokio::test]
async fn script_events_belong_to_dialogue_introductions_only() {
    let server = server();
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    for kind in ["dialogue_exposure", "dialogue_begin"] {
        let body = event(&server, &state, kind, &json!({}));
        assert_eq!(
            submit(&server.router, &state, &[body]).await.status,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}

#[tokio::test]
async fn a_reported_cue_is_ungraded_and_excluded_until_a_new_revision() {
    let server = server();
    let mut content = package();
    ready(&server.router, &mut content).await;
    let state = started(&server.router, &json!({})).await;
    let target = state["manifest"]["targets"][0]["phrase"].clone();
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    let reason = json!({"reason": "This cue is ambiguous"});
    let (_, result) = apply(&server, &result["session"], "report", &reason).await;
    let attempt = &result["session"]["attempts"][0];
    assert_eq!(
        (&attempt["rating"], &attempt["eligibility_reason"]),
        (&Value::Null, &json!("reported_cue"))
    );
    assert_eq!(result["session"]["cursor"], 1);
    let summary = get(&server.router, "/api/v1/sessions/readiness?language=en")
        .await
        .json();
    assert_eq!(readiness(&summary), (2, 1));
    let replacement = started(&server.router, &json!({"takeover_session_id": state["id"]})).await;
    assert!(
        replacement["manifest"]["targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t["phrase"]["id"] != target["id"])
    );
    let unknown = json!({"dialogue_id": Uuid::new_v4(), "takeover_session_id": replacement["id"]});
    assert_eq!(
        start(&server.router, &unknown).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        (
            count(&server, "cue_report"),
            count(&server, "target_schedule")
        ),
        (1, 0)
    );
    content["source_revision"] = json!("2");
    for planned in content["targets"].as_array_mut().unwrap() {
        if planned["source_id"] == target["content"]["source_id"] {
            planned["purpose"] = json!("A clearer purpose.");
        }
    }
    publish(&server.router, &content).await;
    let summary = get(&server.router, "/api/v1/sessions/readiness?language=en")
        .await
        .json();
    assert_eq!(readiness(&summary), (3, 0));
}

#[tokio::test]
async fn delayed_receipts_count_as_support_only_before_the_original_response() {
    for source in ["lookup", "reference"] {
        for before_response in [false, true] {
            for scheduled in [false, true] {
                delayed_support(source, before_response, scheduled).await;
            }
        }
    }
}

async fn delayed_support(source: &str, before_response: bool, scheduled: bool) {
    let case = format!("{source} before={before_response} scheduled={scheduled}");
    let server = server();
    ready(&server.router, &mut package()).await;
    let mut helper = started(&server.router, &json!({})).await;
    if scheduled {
        let (_, presented) = apply(&server, &helper, "present", &json!({})).await;
        let (_, rated) = apply(
            &server,
            &presented["session"],
            "rate",
            &json!({"rating": "recalled"}),
        )
        .await;
        server
            .clock
            .set(moment(rated["receipts"][0]["due_at"].as_str().unwrap()) + TimeDelta::seconds(1));
        helper = started(
            &server.router,
            &json!({"takeover_session_id": helper["id"]}),
        )
        .await;
    }
    let base = server.clock.now();
    let offset = |seconds: i64| timestamps::isoformat(base + TimeDelta::seconds(seconds));
    let helper = apply(&server, &helper, "present", &json!({})).await.1["session"].clone();
    server.clock.set(base + TimeDelta::seconds(1));
    let delayed = started(
        &server.router,
        &json!({"takeover_session_id": helper["id"]}),
    )
    .await;
    let target = delayed["manifest"]["targets"][0]["phrase"].clone();
    assert_eq!(
        target["id"], helper["manifest"]["targets"][0]["phrase"]["id"],
        "{case}"
    );
    server
        .clock
        .set(base + TimeDelta::seconds(if before_response { 3 } else { 5 }));
    if source == "lookup" {
        exposure(&server.router, &target, "reference").await;
    } else {
        apply(&server, &helper, "reference", &json!({})).await;
    }
    server.clock.set(base + TimeDelta::seconds(20));
    let (_, result) = apply(
        &server,
        &delayed,
        "present",
        &json!({"occurred_at": offset(2)}),
    )
    .await;
    assert_eq!(
        result["session"]["attempts"][0]["supported"], false,
        "{case}"
    );
    let (_, result) = apply(
        &server,
        &result["session"],
        "answer",
        &json!({"occurred_at": offset(4)}),
    )
    .await;
    let original = result["session"]["attempts"][0]["original"].clone();
    assert_eq!(original["supported"], before_response, "{case}");
    assert_eq!(result["receipts"][0]["clock_adjusted"], false, "{case}");
    let rating = json!({"rating": "recalled", "occurred_at": offset(6)});
    let (rated, result) = apply(&server, &result["session"], "rate", &rating).await;
    let receipt = &result["receipts"][0];
    assert_eq!(
        result["session"]["attempts"][0]["original"], original,
        "{case}"
    );
    let reason = match (before_response, scheduled) {
        (true, true) => "supported",
        (true, false) => "supported_learning_check",
        (false, _) => "eligible",
    };
    assert_eq!(receipt["eligibility_reason"], reason, "{case}");
    assert_eq!(
        receipt["schedule_changed"],
        !before_response || !scheduled,
        "{case}"
    );
    let version = if before_response || !scheduled { 1 } else { 2 };
    assert_eq!(receipt["schedule_version"], version, "{case}");
    let replay = submit(&server.router, &delayed, &[rated]).await;
    assert_eq!(replay.json()["receipts"], result["receipts"], "{case}");
}

#[tokio::test]
async fn a_delayed_dialogue_counts_its_script_from_when_it_was_shown() {
    let server = server();
    ready(&server.router, &mut package()).await;
    started(&server.router, &json!({})).await;
    let state = dialogue_session(&server).await;
    let shown = server.clock.now();
    server.clock.set(shown + TimeDelta::minutes(1));
    let offset = |seconds: i64| json!({"occurred_at": timestamps::isoformat(shown + TimeDelta::seconds(seconds))});
    let (_, result) = apply(&server, &state, "dialogue_exposure", &offset(1)).await;
    let (_, result) = apply(&server, &result["session"], "dialogue_begin", &offset(2)).await;
    let (_, result) = apply(&server, &result["session"], "present", &offset(3)).await;
    assert_eq!(result["session"]["attempts"][0]["supported"], true);
    let (_, result) = apply(&server, &result["session"], "answer", &offset(4)).await;
    assert_eq!(
        result["session"]["attempts"][0]["original"]["supported"],
        true
    );
    let mut rating = offset(5);
    rating["rating"] = json!("recalled");
    let (_, result) = apply(&server, &result["session"], "rate", &rating).await;
    assert_eq!(
        result["receipts"][0]["eligibility_reason"],
        "supported_learning_check"
    );
}

#[tokio::test]
async fn script_exposure_expires_after_five_minutes_for_later_targets() {
    for (elapsed, supported) in [
        (TimeDelta::minutes(4), true),
        (TimeDelta::minutes(6), false),
    ] {
        let server = server();
        let mut content = package();
        let rather = content["targets"][2].clone();
        content["dialogues"][0]["turns"].as_array_mut().unwrap().push(
            json!({"speaker": "Learner", "text": rather["text"], "target_source_id": rather["source_id"]}),
        );
        ready(&server.router, &mut content).await;
        started(&server.router, &json!({})).await;
        let state = dialogue_session(&server).await;
        let (_, result) = apply(&server, &state, "dialogue_exposure", &json!({})).await;
        let (_, result) = apply(&server, &result["session"], "dialogue_begin", &json!({})).await;
        let (_, result) = apply(&server, &result["session"], "present", &json!({})).await;
        assert_eq!(result["session"]["attempts"][0]["supported"], true);
        let later = server.clock.now() + elapsed;
        server.clock.set(later);
        let recalled = json!({"rating": "recalled"});
        let (_, result) = apply(&server, &result["session"], "rate", &recalled).await;
        let (_, result) = apply(&server, &result["session"], "present", &json!({})).await;
        assert_eq!(result["session"]["attempts"][1]["supported"], supported);
        let (rated, result) = apply(&server, &result["session"], "rate", &recalled).await;
        assert_eq!(
            result["session"]["attempts"][1]["original"]["supported"],
            supported
        );
        let saved = session(&server.router, &state["id"]).await;
        assert_eq!(saved["attempts"][0]["original"]["supported"], true);
        let replay = submit(&server.router, &state, &[rated]).await;
        assert_eq!(replay.json()["receipts"], result["receipts"]);
    }
}
