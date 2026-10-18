use axum::http::StatusCode;
use phrase_gym_core::clock::Clock;
use phrase_gym_core::hashing::sha256_hex;
use phrase_gym_core::timestamps;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::TestServer;
use crate::support::content::{get, package, post};
use crate::support::documents::base64;
use crate::support::practice::{apply, moment, rate, ready, schedule_version, session, started};

async fn due() -> (TestServer, Value) {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    let first = rate(&server, &state, "recalled").await;
    server
        .clock
        .set(moment(first["receipts"][0]["due_at"].as_str().unwrap()));
    let state = started(&server.router, &json!({"takeover_session_id": state["id"]})).await;
    assert_eq!(state["manifest"]["targets"][0]["schedule_version"], 1);
    assert_eq!(state["manifest"]["targets"][0]["eligibility"], "due");
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    (server, result["session"].clone())
}

fn take(server: &TestServer, state: &Value, capture: &Value, complete: bool) -> Value {
    let attempt = &state["attempts"][0];
    json!({
        "id": Uuid::new_v4(), "target_id": attempt["target_id"],
        "revision": attempt["revision"], "attempt_id": attempt["id"],
        "capture_event_id": capture["id"],
        "captured_at": timestamps::isoformat(server.clock.now()),
        "mime_type": "audio/webm", "complete": complete,
        "audio_base64": base64(b"original opaque capture"),
    })
}

async fn store(router: &axum::Router, body: &Value) {
    let reply = post(router, "/api/v1/recordings", body).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    let receipt = reply.json();
    assert_eq!(receipt["complete"], body["complete"]);
    assert_eq!(receipt["sha256"], sha256_hex(b"original opaque capture"));
    assert_eq!(
        post(router, "/api/v1/recordings", body).await.json(),
        receipt
    );
    let bytes = get(
        router,
        &format!("/api/v1/recordings/{}", body["id"].as_str().unwrap()),
    )
    .await;
    assert_eq!(bytes.body.as_ref(), b"original opaque capture");
}

async fn verify_later_capture(original_recorded: bool, ending: &str) {
    let (server, mut state) = due().await;
    let mut takes = Vec::new();
    if original_recorded {
        let (capture, result) =
            apply(&server, &state, "respond", &json!({"modality": "recorded"})).await;
        state = result["session"].clone();
        let body = take(&server, &state, &capture, true);
        store(&server.router, &body).await;
        takes.push(body);
        state = apply(&server, &state, "capture_complete", &json!({}))
            .await
            .1["session"]
            .clone();
    } else {
        state = apply(&server, &state, "answer", &json!({})).await.1["session"].clone();
    }
    let original = state["attempts"][0]["original"].clone();
    assert_eq!(original["supported"], false);
    if ending != "control" {
        let (capture, result) =
            apply(&server, &state, "respond", &json!({"modality": "recorded"})).await;
        state = result["session"].clone();
        if ending == "failure" {
            state = apply(
                &server,
                &state,
                "failure",
                &json!({"reason": "Interrupted rerecord"}),
            )
            .await
            .1["session"]
                .clone();
        } else if ending != "pending" {
            let body = take(&server, &state, &capture, false);
            if ending == "recovered" {
                store(&server.restarted(), &body).await;
            } else {
                store(&server.router, &body).await;
                state = apply(&server, &state, "capture_complete", &json!({}))
                    .await
                    .1["session"]
                    .clone();
            }
            takes.push(body);
        }
    }
    state = session(&server.router, &state["id"]).await;
    let (_, result) = apply(&server, &state, "rate", &json!({"rating": "recalled"})).await;
    let receipt = &result["receipts"][0];
    assert_eq!(
        receipt["eligibility_reason"], "eligible",
        "{original_recorded}/{ending}"
    );
    assert_eq!(receipt["schedule_changed"], true);
    assert_eq!(receipt["schedule_version"], 2);
    let attempt = &result["session"]["attempts"][0];
    assert_eq!(attempt["original"], original);
    assert_eq!(schedule_version(&server, &attempt["target_id"]), Some(2));
    if ending == "failure" {
        let payload: String = rusqlite::Connection::open(&server.database_path)
            .unwrap()
            .query_row(
                "SELECT payload FROM practice_event WHERE attempt_id = ?1 AND kind = 'failure'",
                [attempt["id"].as_str().unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&payload).unwrap()["reason"],
            "Interrupted rerecord"
        );
    }
    for body in &takes {
        store(&server.restarted(), body).await;
    }
}

#[tokio::test]
async fn later_failure_cannot_regrade_a_silent_or_recorded_original() {
    for recorded in [false, true] {
        for ending in ["control", "failure", "pending"] {
            verify_later_capture(recorded, ending).await;
        }
    }
}

#[tokio::test]
async fn later_incomplete_take_cannot_regrade_a_silent_or_recorded_original() {
    for recorded in [false, true] {
        for ending in ["incomplete", "recovered"] {
            verify_later_capture(recorded, ending).await;
        }
    }
}

#[tokio::test]
async fn an_original_capture_failure_stays_excluded_after_a_successful_rerecord() {
    for failure_event in [false, true] {
        let (server, state) = due().await;
        let (capture, result) =
            apply(&server, &state, "respond", &json!({"modality": "recorded"})).await;
        let original_take = take(&server, &result["session"], &capture, false);
        let kind = if failure_event {
            "failure"
        } else {
            "capture_complete"
        };
        let (_, result) = apply(&server, &result["session"], kind, &json!({})).await;
        let (_, result) = apply(
            &server,
            &result["session"],
            "respond",
            &json!({"modality": "recorded"}),
        )
        .await;
        apply(&server, &result["session"], "capture_complete", &json!({})).await;
        if !failure_event {
            store(&server.restarted(), &original_take).await;
        }
        let current = session(&server.router, &state["id"]).await;
        let (_, result) = apply(&server, &current, "rate", &json!({"rating": "recalled"})).await;
        assert_eq!(
            result["receipts"][0]["eligibility_reason"],
            "technical_failure"
        );
        assert_eq!(result["receipts"][0]["schedule_changed"], false);
        assert_eq!(result["receipts"][0]["schedule_version"], 1);
        assert_eq!(
            schedule_version(&server, &current["attempts"][0]["target_id"]),
            Some(1)
        );
    }
}
