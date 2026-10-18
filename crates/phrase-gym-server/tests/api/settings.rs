use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};

use crate::support::content::get;
use crate::support::http::{Reply, send};
use crate::support::practice::moment;
use crate::support::{HOST, TestServer};

async fn put(router: &axum::Router, body: &Value) -> Reply {
    let request = Request::put("/api/v1/settings")
        .header("host", HOST)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap();
    send(router, request).await
}

#[tokio::test]
async fn first_launch_defaults_are_replaced_as_a_whole_and_survive_restart() {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    let initial = get(&server.router, "/api/v1/settings").await;
    assert_eq!(
        initial.json(),
        json!({"native_language": null, "ui_language_mode": "native"})
    );
    let saved = put(
        &server.router,
        &json!({"native_language": "ru", "ui_language_mode": "learning"}),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK);
    assert_eq!(
        saved.json(),
        json!({"native_language": "ru", "ui_language_mode": "learning"})
    );
    let restarted = server.restarted();
    assert_eq!(
        get(&restarted, "/api/v1/settings").await.json(),
        saved.json()
    );
    let replaced = put(&restarted, &json!({"native_language": "sr"})).await;
    assert_eq!(
        replaced.json(),
        json!({"native_language": "sr", "ui_language_mode": "native"})
    );
    assert_eq!(
        get(&restarted, "/api/v1/settings").await.json(),
        replaced.json()
    );
}

#[tokio::test]
async fn incomplete_or_unknown_settings_are_rejected_without_changes() {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    put(&server.router, &json!({"native_language": "ru"})).await;
    for invalid in [
        json!({"native_language": "en", "last_learning_language": "sr"}),
        json!({"ui_language_mode": "learning"}),
        json!({"native_language": "fr"}),
        json!({"native_language": "en", "ui_language_mode": "device"}),
        json!({"native_language": null}),
    ] {
        assert_eq!(
            put(&server.router, &invalid).await.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{invalid}"
        );
    }
    assert_eq!(
        get(&server.router, "/api/v1/settings").await.json(),
        json!({"native_language": "ru", "ui_language_mode": "native"})
    );
    let patch = Request::patch("/api/v1/settings/language")
        .header("host", HOST)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"last_learning_language": "sr"}"#))
        .unwrap();
    assert_eq!(
        send(&server.router, patch).await.status,
        StatusCode::NOT_FOUND
    );
}
