use crate::support::{HOST, TestServer};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::DateTime;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

const TARGET: &str = "2f0c6f1e-5a43-4c1b-9b7e-3d2a1c0e9f10";

fn server() -> TestServer {
    let server = TestServer::start(
        DateTime::parse_from_rfc3339("2026-10-12T12:00:00Z")
            .unwrap()
            .to_utc(),
    );
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .execute_batch(&format!(
            "INSERT INTO target (id, language, kind, meaning_key, recall_expression, current_revision)
             VALUES ('{TARGET}', 'en', 'fixed_phrase', 'ask-day', 'How has your day been so far?', 1);
             INSERT INTO target_revision (target_id, revision, payload, fingerprint, created_at)
             VALUES ('{TARGET}', 1, '{{}}', 'synthetic', '2026-10-01T09:00:00+00:00');"
        ))
        .unwrap();
    server
}

fn exposure(id: &str, kind: &str) -> Value {
    json!({"id": id, "target_id": TARGET, "revision": 1, "language": "en", "kind": kind})
}

async fn post(router: Router, body: Vec<u8>) -> (StatusCode, Value) {
    let request = Request::post("/api/v1/exposures")
        .header("host", HOST)
        .header("authorization", format!("Bearer {}", crate::support::TOKEN))
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn stored_rows(server: &TestServer) -> i64 {
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .query_row("SELECT count(*) FROM exposure", [], |row| row.get(0))
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_identical_requests_commit_one_exposure_and_one_receipt() {
    let server = server();
    let body =
        serde_json::to_vec(&exposure("6a1f3c1e-0d7b-4e43-9a51-3f3e2f6b8a01", "answer")).unwrap();
    let handles: Vec<_> = (0..24)
        .map(|_| tokio::spawn(post(server.router.clone(), body.clone())))
        .collect();
    let mut responses = Vec::new();
    for handle in handles {
        responses.push(handle.await.unwrap());
    }
    assert!(
        responses
            .iter()
            .all(|(status, _)| *status == StatusCode::OK)
    );
    assert!(
        responses
            .iter()
            .all(|(_, receipt)| *receipt == responses[0].1)
    );
    assert_eq!(stored_rows(&server), 1);
}

#[tokio::test]
async fn oversized_bodies_are_rejected_before_parsing() {
    let server = server();
    let mut body = b"{\"id\": \"".to_vec();
    body.resize(24 * 1024 * 1024 + 1, b'a');
    let (status, error) = post(server.router.clone(), body).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(error["detail"].is_string());
    let (status, error) = post(server.router.clone(), b"[]".to_vec()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(error["detail"].is_string());
    assert_eq!(stored_rows(&server), 0);
}
