use axum::http::StatusCode;
use phrase_gym_core::clock::Clock;
use phrase_gym_core::hashing::sha256_hex;
use phrase_gym_core::timestamps;
use serde_json::{Value, json};
use uuid::Uuid;

use super::TestServer;
use super::content::{get, package, post};
use super::documents::base64;
use super::practice::{apply, moment, rate, ready, started};

pub async fn due() -> (TestServer, Value) {
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

pub fn take(server: &TestServer, state: &Value, capture: &Value, complete: bool) -> Value {
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

pub async fn store(router: &axum::Router, body: &Value) {
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
