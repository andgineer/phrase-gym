use std::time::{Duration, Instant};

use axum::http::StatusCode;
use serde_json::json;

use crate::support::TestServer;
use crate::support::capture::{due, take};
use crate::support::content::{get, package, post};
use crate::support::practice::{apply, count, event, moment, ready, started, submit};

#[tokio::test]
async fn pending_delivery_blocks_writes_and_lost_rating_commits_once_before_retry() {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    let (_, presented) = apply(&server, &state, "present", &json!({})).await;
    let state = &presented["session"];
    let rating = event(&server, state, "rate", &json!({"rating": "recalled"}));
    post(&server.router, "/_test/pending-delivery/true", &json!({})).await;
    assert_eq!(
        submit(&server.router, state, std::slice::from_ref(&rating))
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        post(&server.router, "/api/v1/recordings", &json!({}))
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(count(&server, "practice_event"), 1);
    post(&server.router, "/_test/pending-delivery/false", &json!({})).await;
    post(&server.router, "/_test/lose-rating", &json!({})).await;
    assert_eq!(
        submit(&server.router, state, std::slice::from_ref(&rating))
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(count(&server, "review_receipt"), 1);
    post(&server.router, "/_test/acknowledge-rating", &json!({})).await;
    let acknowledged = submit(&server.router, state, std::slice::from_ref(&rating)).await;
    assert_eq!(acknowledged.status, StatusCode::OK);
    assert_eq!(
        submit(&server.router, state, &[rating]).await.json(),
        acknowledged.json()
    );
    assert_eq!(count(&server, "practice_event"), 2);
    assert_eq!(count(&server, "review_receipt"), 1);
    assert_eq!(
        post(&server.router, "/_test/reset", &json!({}))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(count(&server, "target"), 0);
    assert_eq!(count(&server, "practice_session"), 0);
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    apply(&server, &state, "present", &json!({})).await;
}

#[tokio::test]
async fn a_held_session_read_returns_its_snapshot_and_only_holds_one_read() {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    let id = state["id"].as_str().unwrap();
    post(
        &server.router,
        &format!("/_test/hold-session-read/{id}"),
        &json!({}),
    )
    .await;
    let path = format!("/api/v1/sessions/{id}");
    let reader = {
        let router = server.router.clone();
        let path = path.clone();
        tokio::spawn(async move { get(&router, &path).await })
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(
            Instant::now() < deadline,
            "session read did not reach its gate"
        );
        if get(&server.router, "/_test/session-read-state")
            .await
            .json()["held"]
            == true
        {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(!reader.is_finished());
    apply(&server, &state, "present", &json!({})).await;
    assert_eq!(get(&server.router, &path).await.json()["last_sequence"], 1);
    post(&server.router, "/_test/release-session-read", &json!({})).await;
    assert_eq!(reader.await.unwrap().json(), state);
    assert_eq!(
        get(&server.router, "/_test/session-read-state")
            .await
            .json()["held"],
        false
    );
}

#[tokio::test]
async fn held_recording_ack_keeps_committed_bytes_readable_and_reset_releases_it() {
    let (server, state) = due().await;
    let (capture, responded) =
        apply(&server, &state, "respond", &json!({"modality": "recorded"})).await;
    let body = take(&server, &responded["session"], &capture, false);
    post(&server.router, "/_test/hold-recording-ack", &json!({})).await;
    let writer = {
        let router = server.router.clone();
        let body = body.clone();
        tokio::spawn(async move { post(&router, "/api/v1/recordings", &body).await })
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    while count(&server, "learner_recording") == 0 {
        assert!(Instant::now() < deadline, "recording did not commit");
        tokio::task::yield_now().await;
    }
    assert!(!writer.is_finished());
    let path = format!("/api/v1/recordings/{}", body["id"].as_str().unwrap());
    assert_eq!(
        get(&server.router, &path).await.body.as_ref(),
        b"original opaque capture"
    );
    post(&server.router, "/_test/reset", &json!({})).await;
    assert_eq!(writer.await.unwrap().status, StatusCode::OK);
    assert_eq!(
        get(&server.router, &path).await.status,
        StatusCode::NOT_FOUND
    );
}
