use axum::Router;
use axum::http::StatusCode;
use chrono::TimeDelta;
use serde_json::{Value, json};

use crate::support::TestServer;
use crate::support::capture::{due, store, take};
use crate::support::content::get;
use crate::support::practice::{advance, event, schedule_version, session, submit};

async fn accept(
    server: &TestServer,
    router: &Router,
    state: &Value,
    kind: &str,
    extra: &Value,
) -> (Value, Value) {
    let body = event(server, state, kind, extra);
    let reply = submit(router, state, std::slice::from_ref(&body)).await;
    assert_eq!(reply.status, StatusCode::OK, "{kind}: {:?}", reply.body);
    (body, reply.json())
}

#[tokio::test]
async fn original_completion_after_restart_and_identical_retry_remains_eligible() {
    for retry in [false, true] {
        let (server, state) = due().await;
        let (capture, result) = accept(
            &server,
            &server.router,
            &state,
            "respond",
            &json!({"modality": "recorded"}),
        )
        .await;
        let original = result["session"]["attempts"][0]["original"].clone();
        let restarted = server.restarted();
        let mut state = session(&restarted, &state["id"]).await;
        assert_eq!(state["attempts"][0]["original"], original);
        assert_eq!(state["attempts"][0]["capture_pending"], true);
        advance(&server, TimeDelta::seconds(30));
        if retry {
            let reply = submit(&restarted, &state, std::slice::from_ref(&capture)).await;
            assert_eq!(reply.status, StatusCode::OK);
            let retried = reply.json();
            assert_eq!(retried["receipts"], result["receipts"]);
            assert_eq!(retried["session"], state);
            state = retried["session"].clone();
        }
        store(&restarted, &take(&server, &state, &capture, true)).await;
        let (_, result) = accept(&server, &restarted, &state, "capture_complete", &json!({})).await;
        let (_, result) = accept(
            &server,
            &restarted,
            &result["session"],
            "rate",
            &json!({"rating": "recalled"}),
        )
        .await;
        assert_eq!(result["receipts"][0]["eligibility_reason"], "eligible");
        assert_eq!(result["receipts"][0]["schedule_changed"], true);
        assert_eq!(result["receipts"][0]["schedule_version"], 2);
        assert_eq!(result["session"]["attempts"][0]["original"], original);
        assert_eq!(
            schedule_version(&server, &state["attempts"][0]["target_id"]),
            Some(2)
        );
        let events = get(
            &restarted,
            &format!("/api/v1/sessions/{}/events", state["id"].as_str().unwrap()),
        )
        .await
        .json();
        assert_eq!(events.as_array().unwrap().len(), 4);
    }
}

#[tokio::test]
async fn distinct_captures_cannot_rescue_an_abandoned_original_after_restart() {
    for (rerecords, upload_after_rating) in [(1, false), (3, false), (3, true)] {
        let (server, state) = due().await;
        let (capture, result) = accept(
            &server,
            &server.router,
            &state,
            "respond",
            &json!({"modality": "recorded"}),
        )
        .await;
        let original = result["session"]["attempts"][0]["original"].clone();
        assert_eq!(original["modality"], "recorded");
        assert_eq!(original["supported"], false);
        assert_eq!(
            original["occurred_at"],
            result["receipts"][0]["occurred_at"]
        );
        let original_take = take(&server, &result["session"], &capture, true);
        let restarted = server.restarted();
        let mut state = session(&restarted, &state["id"]).await;
        assert_eq!(state["attempts"][0]["capture_pending"], true);
        assert_eq!(state["attempts"][0]["original"], original);
        for _ in 0..rerecords {
            advance(&server, TimeDelta::seconds(30));
            let (later, result) = accept(
                &server,
                &restarted,
                &state,
                "respond",
                &json!({"modality": "recorded"}),
            )
            .await;
            store(&restarted, &take(&server, &result["session"], &later, true)).await;
            state = accept(
                &server,
                &restarted,
                &result["session"],
                "capture_complete",
                &json!({}),
            )
            .await
            .1["session"]
                .clone();
        }
        if !upload_after_rating {
            store(&restarted, &original_take).await;
            state = accept(&server, &restarted, &state, "capture_complete", &json!({}))
                .await
                .1["session"]
                .clone();
        }
        let (_, result) = accept(
            &server,
            &restarted,
            &state,
            "rate",
            &json!({"rating": "recalled"}),
        )
        .await;
        assert_eq!(
            result["receipts"][0]["eligibility_reason"],
            "technical_failure"
        );
        assert_eq!(result["receipts"][0]["schedule_changed"], false);
        assert_eq!(result["receipts"][0]["schedule_version"], 1);
        assert_eq!(result["session"]["attempts"][0]["original"], original);
        store(&server.restarted(), &original_take).await;
        let restored = session(&server.restarted(), &state["id"]).await;
        assert_eq!(restored["attempts"][0], result["session"]["attempts"][0]);
        assert_eq!(
            schedule_version(&server, &state["attempts"][0]["target_id"]),
            Some(1)
        );
        let retained = get(
            &restarted,
            &format!(
                "/api/v1/phrases/{}/recordings",
                state["attempts"][0]["target_id"].as_str().unwrap()
            ),
        )
        .await
        .json();
        assert_eq!(retained.as_array().unwrap().len(), rerecords + 1);
    }
}
