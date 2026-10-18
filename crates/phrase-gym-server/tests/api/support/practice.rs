use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, TimeDelta, Utc};
use phrase_gym_core::clock::Clock;
use phrase_gym_core::timestamps;
use rusqlite::OptionalExtension;
use serde_json::{Value, json};
use uuid::Uuid;

use super::content::{WAV, get, phrases, post, publish, upload};
use super::http::{Reply, send};
use super::{HOST, TestServer};

pub fn moment(text: &str) -> DateTime<Utc> {
    timestamps::parse(text).unwrap()
}

pub fn at(server: &TestServer, offset: TimeDelta) -> String {
    timestamps::isoformat(server.clock.now() + offset)
}

pub fn advance(server: &TestServer, by: TimeDelta) -> DateTime<Utc> {
    let later = server.clock.now() + by;
    server.clock.set(later);
    later
}

// Uploads reference audio for every target, publishes the package and returns its phrases.
pub async fn ready(router: &Router, package: &mut Value) -> Vec<Value> {
    for target in package["targets"].as_array_mut().unwrap() {
        let asset = upload(router, &json!({"text": target["text"]}), WAV, "audio/wav").await;
        target["reference_audio_id"] = asset["id"].clone();
    }
    publish(router, package).await;
    phrases(router, "en", "").await
}

pub async fn start(router: &Router, parameters: &Value) -> Reply {
    let mut body = json!({"id": Uuid::new_v4(), "language": "en"});
    for (key, value) in parameters.as_object().unwrap() {
        body[key] = value.clone();
    }
    post(router, "/api/v1/sessions", &body).await
}

pub async fn started(router: &Router, parameters: &Value) -> Value {
    let reply = start(router, parameters).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    reply.json()
}

pub async fn session(router: &Router, id: &Value) -> Value {
    let reply = get(
        router,
        &format!("/api/v1/sessions/{}", id.as_str().unwrap()),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    reply.json()
}

pub async fn current(router: &Router) -> Value {
    get(router, "/api/v1/sessions/current").await.json()
}

pub async fn patch(router: &Router, id: &Value, body: &Value) -> Reply {
    let request = Request::patch(format!("/api/v1/sessions/{}", id.as_str().unwrap()))
        .header("host", HOST)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap();
    send(router, request).await
}

// The next event for the session's current step at the server clock, with optional overrides.
pub fn event(server: &TestServer, state: &Value, kind: &str, extra: &Value) -> Value {
    let mut body = json!({
        "id": Uuid::new_v4(),
        "sequence": state["last_sequence"].as_i64().unwrap() + 1,
        "step": state["cursor"],
        "kind": kind,
        "occurred_at": timestamps::isoformat(server.clock.now()),
    });
    for (key, value) in extra.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

pub async fn submit(router: &Router, state: &Value, events: &[Value]) -> Reply {
    let path = format!("/api/v1/sessions/{}/events", state["id"].as_str().unwrap());
    post(router, &path, &json!({"events": events})).await
}

pub async fn apply(
    server: &TestServer,
    state: &Value,
    kind: &str,
    extra: &Value,
) -> (Value, Value) {
    let body = event(server, state, kind, extra);
    let reply = submit(&server.router, state, std::slice::from_ref(&body)).await;
    assert_eq!(reply.status, StatusCode::OK, "{kind}: {:?}", reply.body);
    (body, reply.json())
}

// Presents and rates the current card, returning the batch result.
pub async fn rate(server: &TestServer, state: &Value, rating: &str) -> Value {
    let (_, presented) = apply(server, state, "present", &json!({})).await;
    apply(
        server,
        &presented["session"],
        "rate",
        &json!({"rating": rating}),
    )
    .await
    .1
}

pub async fn exposure(router: &Router, phrase: &Value, kind: &str) -> Value {
    let id = Uuid::new_v4();
    let reply = post(
        router,
        "/api/v1/exposures",
        &json!({
            "id": id, "target_id": phrase["id"], "revision": phrase["revision"],
            "language": "en", "kind": kind,
        }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    json!(id)
}

pub fn count(server: &TestServer, table: &str) -> i64 {
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

pub fn execute(server: &TestServer, sql: &str) {
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .execute_batch(sql)
        .unwrap();
}

pub fn schedule_version(server: &TestServer, target: &Value) -> Option<i64> {
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .query_row(
            "SELECT version FROM target_schedule WHERE target_id = ?1",
            [target.as_str().unwrap()],
            |row| row.get(0),
        )
        .optional()
        .unwrap()
}
