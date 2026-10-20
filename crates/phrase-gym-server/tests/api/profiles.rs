use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::DateTime;
use serde_json::{Value, json};

use crate::support::content::get;
use crate::support::http::{Reply, send};
use crate::support::{HOST, TestServer};

async fn put(server: &TestServer, path: &str, value: &Value) -> Reply {
    send(
        &server.router,
        Request::put(path)
            .header("host", HOST)
            .header("content-type", "application/json")
            .body(Body::from(value.to_string()))
            .unwrap(),
    )
    .await
}

#[tokio::test]
async fn profile_api_preserves_authorized_revision_receipts_independently_of_device_settings() {
    let server = TestServer::start(
        DateTime::parse_from_rfc3339("2026-10-19T08:00:00Z")
            .unwrap()
            .to_utc(),
    );
    let first = get(&server.router, "/api/v1/profile").await.json();
    assert_eq!(first["profile"]["native_language"], "ru");
    let device = put(
        &server,
        "/api/v1/settings",
        &json!({"native_language": "en", "ui_language_mode": "learning"}),
    )
    .await;
    assert_eq!(device.status, StatusCode::OK);
    assert_eq!(get(&server.router, "/api/v1/profile").await.json(), first);
    let mut profile = first["profile"].clone();
    profile["learning_timezone"] = json!("UTC");
    let update = json!({"mutation_id": uuid::Uuid::new_v4(), "expected_revision": 1, "profile": profile, "reason": "Authorized learner request"});
    let second = put(&server, "/api/v1/profile", &update).await;
    assert_eq!(second.status, StatusCode::OK);
    assert_eq!(second.json()["actor"], "private-operator");
    assert_eq!(second.json()["revision"], 2);
    assert_eq!(
        put(&server, "/api/v1/profile", &update).await.json(),
        second.json()
    );
    let mut conflict = update.clone();
    conflict["mutation_id"] = json!(uuid::Uuid::new_v4());
    assert_eq!(
        put(&server, "/api/v1/profile", &conflict).await.status,
        StatusCode::CONFLICT
    );
    let stored = get(&server.restarted(), "/api/v1/profile").await;
    assert_eq!(stored.json(), second.json());
    assert_eq!(
        get(&server.router, "/api/v1/profile/revisions/1")
            .await
            .json(),
        first
    );
}
