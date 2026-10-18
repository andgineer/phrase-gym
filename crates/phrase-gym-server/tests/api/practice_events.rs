use axum::http::StatusCode;
use chrono::TimeDelta;
use phrase_gym_core::clock::Clock;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::TestServer;
use crate::support::content::{get, package, publish};
use crate::support::practice::{
    advance, apply, at, count, current, event, exposure, moment, rate, ready, session, started,
    submit,
};

async fn practising() -> (TestServer, Value) {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    (server, state)
}

fn receipt(result: &Value) -> &Value {
    &result["receipts"][0]
}

#[tokio::test]
async fn eligible_rating_is_idempotent_snapshots_the_original_and_completes() {
    let (server, state) = practising().await;
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "answer", &json!({})).await;
    assert_eq!(
        result["session"]["attempts"][0]["original"]["supported"],
        false
    );
    let recorded = json!({"modality": "recorded"});
    let (_, result) = apply(&server, &result["session"], "respond", &recorded).await;
    assert_eq!(
        result["session"]["attempts"][0]["original"]["modality"],
        "self_report"
    );
    let (_, result) = apply(&server, &result["session"], "capture_complete", &json!({})).await;
    let effort = json!({"rating": "with_effort"});
    let (rated, result) = apply(&server, &result["session"], "rate", &effort).await;
    assert_eq!(receipt(&result)["schedule_changed"], true);
    assert_eq!(receipt(&result)["eligibility_reason"], "eligible");
    assert!(moment(receipt(&result)["due_at"].as_str().unwrap()) > server.clock.now());
    let replay = submit(&server.router, &state, std::slice::from_ref(&rated)).await;
    assert_eq!(replay.json()["receipts"], result["receipts"]);
    let mut changed = rated.clone();
    changed["rating"] = json!("recalled");
    assert_eq!(
        submit(&server.router, &state, &[changed]).await.status,
        StatusCode::CONFLICT
    );
    let mut state = result["session"].clone();
    for _ in 0..2 {
        state = rate(&server, &state, "recalled").await["session"].clone();
    }
    assert_eq!(state["status"], "completed");
    assert_eq!(current(&server.router).await, Value::Null);
    assert_eq!(count(&server, "review_receipt"), 3);
    let versions: i64 = rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .query_row(
            "SELECT count(*) FROM target_schedule WHERE version = 1 AND memory_state IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(versions, 3);
}

#[tokio::test]
async fn help_before_the_response_is_not_recall_and_earns_a_next_day_check() {
    let (server, state) = practising().await;
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "explain", &json!({})).await;
    let recorded = json!({"modality": "recorded"});
    let (_, result) = apply(&server, &result["session"], "respond", &recorded).await;
    let (_, result) = apply(&server, &result["session"], "capture_complete", &json!({})).await;
    let recalled = json!({"rating": "recalled"});
    let (_, result) = apply(&server, &result["session"], "rate", &recalled).await;
    let attempt = &result["session"]["attempts"][0];
    assert_eq!(
        (&attempt["original"]["supported"], &attempt["rating"]),
        (&json!(true), &json!("recalled"))
    );
    assert_eq!(
        receipt(&result)["eligibility_reason"],
        "supported_learning_check"
    );
    assert_eq!(
        moment(receipt(&result)["due_at"].as_str().unwrap()),
        server.clock.now() + TimeDelta::days(1)
    );
}

#[tokio::test]
async fn a_sequence_gap_rolls_back_the_batch_and_implausible_times_are_bounded() {
    let (server, state) = practising().await;
    let present = event(&server, &state, "present", &json!({}));
    let mut gap = event(&server, &state, "answer", &json!({"sequence": 3}));
    gap["step"] = json!(0);
    assert_eq!(
        submit(&server.router, &state, &[present, gap]).await.status,
        StatusCode::CONFLICT
    );
    let state = session(&server.router, &state["id"]).await;
    assert_eq!(
        (&state["last_sequence"], &state["attempts"]),
        (&json!(0), &json!([]))
    );
    let future = json!({"occurred_at": at(&server, TimeDelta::days(100))});
    let (_, result) = apply(&server, &state, "present", &future).await;
    assert_eq!(receipt(&result)["clock_adjusted"], true);
    let past = json!({"rating": "recalled", "occurred_at": at(&server, -TimeDelta::days(100))});
    let (_, result) = apply(&server, &result["session"], "rate", &past).await;
    assert_eq!(receipt(&result)["clock_adjusted"], true);
    assert_eq!(
        moment(receipt(&result)["occurred_at"].as_str().unwrap()),
        server.clock.now()
    );
    let path = format!("/api/v1/sessions/{}/events", state["id"].as_str().unwrap());
    assert_eq!(
        get(&server.router, &path)
            .await
            .json()
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn client_time_within_five_minutes_of_skew_is_kept_and_never_runs_backwards() {
    let (server, state) = practising().await;
    let start = server.clock.now();
    advance(&server, TimeDelta::minutes(10));
    let ahead = json!({"occurred_at": at(&server, TimeDelta::minutes(5))});
    let (_, result) = apply(&server, &state, "present", &ahead).await;
    assert_eq!(receipt(&result)["clock_adjusted"], true);
    assert_eq!(
        moment(receipt(&result)["occurred_at"].as_str().unwrap()),
        server.clock.now()
    );
    let before_start = json!({"occurred_at": timestamps(start - TimeDelta::minutes(5))});
    let (_, result) = apply(&server, &result["session"], "answer", &before_start).await;
    assert_eq!(receipt(&result)["clock_adjusted"], true);
    assert_eq!(
        moment(receipt(&result)["occurred_at"].as_str().unwrap()),
        server.clock.now(),
        "corrected time never precedes the previous receipt"
    );
    let replacement = started(&server.router, &json!({"takeover_session_id": state["id"]})).await;
    let honest = json!({"occurred_at": at(&server, -TimeDelta::seconds(30))});
    let (_, result) = apply(&server, &replacement, "present", &honest).await;
    assert_eq!(
        receipt(&result)["clock_adjusted"],
        true,
        "a new session starts at its own time"
    );
    let skewed = json!({"occurred_at": at(&server, TimeDelta::minutes(5) + TimeDelta::seconds(1))});
    let (_, result) = apply(&server, &result["session"], "answer", &skewed).await;
    assert_eq!(receipt(&result)["clock_adjusted"], true);
    assert_eq!(
        moment(receipt(&result)["occurred_at"].as_str().unwrap()),
        server.clock.now()
    );
    let (_, result) = apply(&server, &result["session"], "explain", &json!({})).await;
    assert_eq!(receipt(&result)["clock_adjusted"], false);
}

fn timestamps(moment: chrono::DateTime<chrono::Utc>) -> String {
    phrase_gym_core::timestamps::isoformat(moment)
}

#[tokio::test]
async fn ratings_on_a_replaced_session_stay_history_without_schedule_changes() {
    let (server, state) = practising().await;
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    let replacement = started(&server.router, &json!({"takeover_session_id": state["id"]})).await;
    let recalled = json!({"rating": "recalled"});
    let (_, result) = apply(&server, &result["session"], "rate", &recalled).await;
    assert_eq!(receipt(&result)["eligibility_reason"], "closed_session");
    assert_eq!(receipt(&result)["schedule_changed"], false);
    assert_eq!(result["session"]["status"], "superseded");
    assert_eq!(current(&server.router).await["id"], replacement["id"]);
    assert_eq!(count(&server, "target_schedule"), 0);
}

#[tokio::test]
async fn stale_content_and_technical_failure_never_grade_forgetting() {
    let (server, state) = practising().await;
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    let source = state["manifest"]["targets"][0]["phrase"]["content"]["source_id"].clone();
    let mut revised = package();
    revised["source_revision"] = json!("2");
    for target in revised["targets"].as_array_mut().unwrap() {
        target["reference_audio_id"] = state["manifest"]["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|planned| planned["phrase"]["content"]["source_id"] == target["source_id"])
            .unwrap()["phrase"]["content"]["reference_audio_id"]
            .clone();
        if target["source_id"] == source {
            target["purpose"] = json!("Updated purpose");
        }
    }
    publish(&server.router, &revised).await;
    let failed = json!({"rating": "not_yet"});
    let (_, result) = apply(&server, &result["session"], "rate", &failed).await;
    assert_eq!(receipt(&result)["eligibility_reason"], "stale_content");
    let (_, result) = apply(&server, &result["session"], "present", &json!({})).await;
    let interrupted = json!({"reason": "Capture interrupted"});
    let (_, result) = apply(&server, &result["session"], "failure", &interrupted).await;
    let (_, result) = apply(&server, &result["session"], "rate", &failed).await;
    assert_eq!(receipt(&result)["eligibility_reason"], "technical_failure");
    assert_eq!(receipt(&result)["schedule_changed"], false);
    assert_eq!(count(&server, "target_schedule"), 0);
}

#[tokio::test]
async fn lookups_before_or_during_the_card_count_as_support() {
    for lookup_after_present in [false, true] {
        let (server, state) = practising().await;
        let phrase = &state["manifest"]["targets"][0]["phrase"];
        if !lookup_after_present {
            exposure(&server.router, phrase, "explanation").await;
        }
        let (_, result) = apply(&server, &state, "present", &json!({})).await;
        if lookup_after_present {
            exposure(&server.router, phrase, "explanation").await;
            let (_, answered) = apply(&server, &result["session"], "answer", &json!({})).await;
            assert_eq!(
                answered["session"]["attempts"][0]["original"]["supported"],
                true
            );
        }
        let current = session(&server.router, &state["id"]).await;
        let (_, rated) = apply(&server, &current, "rate", &json!({"rating": "recalled"})).await;
        assert_eq!(
            rated["session"]["attempts"][0]["original"]["supported"],
            true
        );
        assert_eq!(
            receipt(&rated)["eligibility_reason"],
            "supported_learning_check"
        );
    }
}

#[tokio::test]
async fn support_after_the_original_response_never_changes_it() {
    for kind in [
        "lookup",
        "answer",
        "explain",
        "reference",
        "capture_complete",
    ] {
        let (server, state) = practising().await;
        let (_, result) = apply(&server, &state, "present", &json!({})).await;
        let mut state = result["session"].clone();
        advance(&server, TimeDelta::minutes(1));
        if kind == "lookup" {
            exposure(
                &server.router,
                &state["manifest"]["targets"][0]["phrase"],
                "reference",
            )
            .await;
        } else {
            if kind == "capture_complete" {
                state = apply(&server, &state, "respond", &json!({})).await.1["session"].clone();
            }
            state = apply(&server, &state, kind, &json!({})).await.1["session"].clone();
        }
        advance(&server, TimeDelta::minutes(6));
        let current = session(&server.router, &state["id"]).await;
        let (_, rated) = apply(&server, &current, "rate", &json!({"rating": "recalled"})).await;
        let original = &rated["session"]["attempts"][0]["original"];
        assert_eq!(
            original["supported"],
            json!(matches!(kind, "explain" | "reference")),
            "{kind}"
        );
        assert_eq!(
            &session(&server.router, &state["id"]).await["attempts"][0]["original"],
            original
        );
    }
}

#[tokio::test]
async fn help_counts_from_its_actual_time_for_five_minutes() {
    for kind in ["answer", "explain", "reference", "capture_complete"] {
        for expired in [false, true] {
            let (server, state) = practising().await;
            let (_, result) = apply(&server, &state, "present", &json!({})).await;
            let mut current = result["session"].clone();
            if kind == "capture_complete" {
                current =
                    apply(&server, &current, "respond", &json!({})).await.1["session"].clone();
            }
            advance(&server, TimeDelta::minutes(10));
            let (_, helped) = apply(&server, &current, kind, &json!({})).await;
            let original = helped["session"]["attempts"][0]["original"].clone();
            advance(&server, TimeDelta::minutes(if expired { 6 } else { 0 }));
            let next = started(&server.router, &json!({"takeover_session_id": state["id"]})).await;
            assert_eq!(
                next["manifest"]["targets"][0]["phrase"]["id"],
                state["manifest"]["targets"][0]["phrase"]["id"]
            );
            let (_, presented) = apply(&server, &next, "present", &json!({})).await;
            assert_eq!(
                presented["session"]["attempts"][0]["supported"],
                json!(!expired),
                "{kind}"
            );
            let (_, rated) = apply(
                &server,
                &presented["session"],
                "rate",
                &json!({"rating": "recalled"}),
            )
            .await;
            let expected = if expired {
                "eligible"
            } else {
                "supported_learning_check"
            };
            assert_eq!(
                receipt(&rated)["eligibility_reason"],
                expected,
                "{kind} {expired}"
            );
            assert_eq!(
                session(&server.router, &state["id"]).await["attempts"][0]["original"],
                original
            );
        }
    }
}

#[tokio::test]
async fn events_require_a_presented_card_and_valid_kinds() {
    let (server, state) = practising().await;
    let early = event(&server, &state, "answer", &json!({}));
    assert_eq!(
        submit(&server.router, &state, &[early]).await.status,
        StatusCode::CONFLICT
    );
    for invalid in [
        json!({"kind": "rate"}),
        json!({"kind": "present", "rating": "recalled"}),
        json!({"kind": "report"}),
        json!({"kind": "report", "reason": ""}),
        json!({"kind": "present", "occurred_at": "2026-10-12T12:00:00"}),
        json!({"kind": "present", "defer_repairs": true}),
        json!({"kind": "present", "reason": "x".repeat(1001)}),
    ] {
        let mut body = event(&server, &state, "present", &json!({}));
        for (key, value) in invalid.as_object().unwrap() {
            body[key] = value.clone();
        }
        let reply = submit(&server.router, &state, &[body]).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{invalid}");
    }
    let empty = submit(&server.router, &state, &[]).await;
    assert_eq!(empty.status, StatusCode::UNPROCESSABLE_ENTITY);
    let other_session = json!({"id": Uuid::new_v4()});
    let unknown = submit(
        &server.router,
        &other_session,
        &[event(&server, &state, "present", &json!({}))],
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(count(&server, "practice_event"), 0);
}

#[tokio::test]
async fn an_event_id_belongs_to_one_session_and_payload() {
    let (server, state) = practising().await;
    let (present, result) = apply(&server, &state, "present", &json!({})).await;
    let mut rewritten = present.clone();
    rewritten["occurred_at"] = json!(
        phrase_gym_core::timestamps::isoformat(moment(present["occurred_at"].as_str().unwrap()))
            .replace("+00:00", "Z")
    );
    let same_instant = submit(&server.router, &state, std::slice::from_ref(&rewritten)).await;
    assert_eq!(same_instant.json()["receipts"], result["receipts"]);
    let replacement = started(&server.router, &json!({"takeover_session_id": state["id"]})).await;
    let moved = submit(&server.router, &replacement, &[present]).await;
    assert_eq!(moved.status, StatusCode::CONFLICT);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_identical_batches_commit_one_event_and_one_receipt() {
    let (server, state) = practising().await;
    let body = event(&server, &state, "present", &json!({}));
    let tasks: Vec<_> = (0..6)
        .map(|_| {
            let (router, state, body) = (server.router.clone(), state.clone(), body.clone());
            tokio::spawn(async move { submit(&router, &state, &[body]).await })
        })
        .collect();
    let mut receipts = Vec::new();
    for task in tasks {
        let reply = task.await.unwrap();
        assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
        receipts.push(reply.json()["receipts"][0].clone());
    }
    assert!(receipts.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(count(&server, "practice_event"), 1);
    assert_eq!(count(&server, "practice_attempt"), 1);
}
