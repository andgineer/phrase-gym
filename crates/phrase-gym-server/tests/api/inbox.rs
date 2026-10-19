use crate::support::documents::base64;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::DateTime;
use phrase_gym_core::clock::Clock;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::content::get;
use crate::support::http::{Reply, send};
use crate::support::{HOST, TestServer};

async fn post(app: &Router, path: &str, payload: &Value) -> Reply {
    send(
        app,
        Request::post(path)
            .header("host", HOST)
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    )
    .await
}

async fn lesson(server: &TestServer) -> (Value, Value) {
    let media = post(&server.router, "/api/v1/media", &json!({
        "mime_type": "audio/wav", "audio_base64": base64(include_bytes!("../../../../fixtures/preservation/media/silence-100ms.wav")),
        "metadata": {"language": "en", "text": "Let's get started.", "role": "reference", "origin": "course", "source": "synthetic", "performer": "Synthetic", "locale": "en-US", "preparation_version": "synthetic-v1"}
    })).await;
    assert_eq!(media.status, StatusCode::OK, "{}", media.json());
    let media = media.json();
    let input = json!({"submission_id": Uuid::new_v4(), "item_id": "meeting", "expected_generation": null, "language": "en", "title": "Opening a meeting", "context": "All participants are present.", "note": "", "phrases": [{"source_id": "open", "text": "Let's get started.", "reference_audio_id": media["id"], "audio_sha256": media["sha256"], "expected_revision": null}]});
    (input, media)
}

fn output(claim: &Value) -> Value {
    json!({"fence": claim["fence"], "publication_id": Uuid::new_v4(), "adapter": "synthetic-fake-worker-v1", "package": {
        "schema_version": "english_phrases_v1", "namespace": claim["receipt"]["namespace"], "source_revision": claim["receipt"]["generation"].to_string(), "language": "en", "profile_revision": claim["receipt"]["profile_revision"], "guide_version": claim["receipt"]["guide_version"], "assessment_version": "situation-v1",
        "lesson": {"source_id": "meeting", "title": "Opening a meeting", "target_source_ids": ["open"]},
        "phrases": [{"source_id": "open", "text": "Let's get started.", "meaning_key": "begin-activity", "intention_key": "open-meeting", "accepted_variants": [], "situations": [{"id": "call", "text": "Everyone has arrived. Open the meeting.", "language": "en", "audio_id": null, "partner": null, "exposed_target_source_ids": []}], "hint": "started", "help": {"language": "en", "meaning": "Begin a shared activity.", "usage": null, "translation": null}, "familiarity": {"kind": "new"}, "reference_audio_id": claim["input"]["phrases"][0]["reference_audio_id"], "expected_revision": claim["input"]["phrases"][0]["expected_revision"]}]
    }})
}

async fn accept_and_claim(server: &TestServer, input: &Value) -> (Router, String, Value) {
    let accepted = post(&server.router, "/api/v1/inbox/lessons", input).await;
    assert_eq!(accepted.status, StatusCode::OK);
    let receipt = accepted.json();
    let restarted = server.restarted();
    assert_eq!(
        post(&restarted, "/api/v1/inbox/lessons", input)
            .await
            .json(),
        receipt
    );
    let status_path = format!(
        "/api/v1/inbox/lessons/{}",
        receipt["generation_id"].as_str().unwrap()
    );
    assert_eq!(
        get(&restarted, &status_path).await.json()["status"],
        "waiting"
    );
    let claim_id = json!({"claim_id": Uuid::new_v4()});
    let claimed = post(&restarted, "/api/v1/preparation/claims", &claim_id).await;
    assert_eq!(claimed.status, StatusCode::OK);
    let claim = claimed.json();
    assert_eq!(&claim["input"], input);
    assert_eq!(
        post(&restarted, "/api/v1/preparation/claims", &claim_id)
            .await
            .json(),
        claim
    );
    (restarted, status_path, claim)
}

#[tokio::test]
async fn fake_worker_publishes_through_production_routes_and_recovers_receipts_across_restart() {
    let server = TestServer::start(
        DateTime::parse_from_rfc3339("2026-10-19T10:00:00Z")
            .unwrap()
            .to_utc(),
    );
    let (input, media) = lesson(&server).await;
    let (restarted, status_path, claim) = accept_and_claim(&server, &input).await;
    let downloaded = post(
        &restarted,
        "/api/v1/preparation/audio",
        &json!({"fence": claim["fence"], "media_id": media["id"]}),
    )
    .await;
    assert_eq!(downloaded.status, StatusCode::OK);
    assert_eq!(
        downloaded.body.as_ref(),
        include_bytes!("../../../../fixtures/preservation/media/silence-100ms.wav")
    );
    let output = output(&claim);
    assert_eq!(
        post(&restarted, "/api/v1/preparation/stage", &output)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        get(&restarted, &status_path).await.json()["status"],
        "pending_delivery"
    );
    let published = post(&restarted, "/api/v1/preparation/publish", &claim["fence"]).await;
    assert_eq!(published.status, StatusCode::OK);
    let published = published.json();
    server
        .clock
        .set(server.clock.now() + chrono::Duration::seconds(1000));
    let restarted = server.restarted();
    assert_eq!(
        post(&restarted, "/api/v1/preparation/receipt", &claim["fence"])
            .await
            .json(),
        published
    );
    assert_eq!(
        post(&restarted, "/api/v1/preparation/publish", &claim["fence"])
            .await
            .json(),
        published
    );
    assert_eq!(
        post(&restarted, "/api/v1/preparation/stage", &output)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        get(&restarted, &status_path).await.json()["status"],
        "published"
    );
    let phrases = get(
        &restarted,
        &format!(
            "/api/v1/lessons/{}/phrases?language=en",
            published["lesson_id"].as_str().unwrap()
        ),
    )
    .await;
    assert_eq!(phrases.status, StatusCode::OK);
    assert_eq!(phrases.json()["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        get(
            &restarted,
            &format!("/api/v1/media/{}", media["id"].as_str().unwrap())
        )
        .await
        .body
        .as_ref(),
        downloaded.body.as_ref()
    );
    let mut conflict = input.clone();
    conflict["note"] = json!("Changed input");
    assert_eq!(
        post(&restarted, "/api/v1/inbox/lessons", &conflict)
            .await
            .status,
        StatusCode::CONFLICT
    );
    let mut conflict = output.clone();
    conflict["package"]["lesson"]["title"] = json!("Changed output");
    assert_eq!(
        post(&restarted, "/api/v1/preparation/stage", &conflict)
            .await
            .status,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn expired_and_cancelled_http_workers_cannot_publish_or_read_unrelated_audio() {
    let server = TestServer::start(
        DateTime::parse_from_rfc3339("2026-10-19T10:00:00Z")
            .unwrap()
            .to_utc(),
    );
    let (input, _) = lesson(&server).await;
    let receipt = post(&server.router, "/api/v1/inbox/lessons", &input)
        .await
        .json();
    let claim = post(
        &server.router,
        "/api/v1/preparation/claims",
        &json!({"claim_id": Uuid::new_v4()}),
    )
    .await
    .json();
    assert_eq!(
        post(
            &server.router,
            "/api/v1/preparation/audio",
            &json!({"fence": claim["fence"], "media_id": Uuid::new_v4()})
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let output = output(&claim);
    assert_eq!(
        post(&server.router, "/api/v1/preparation/stage", &output)
            .await
            .status,
        StatusCode::OK
    );
    server
        .clock
        .set(server.clock.now() + chrono::Duration::seconds(120));
    assert_eq!(
        post(
            &server.router,
            "/api/v1/preparation/publish",
            &claim["fence"]
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    let fresh = post(
        &server.router,
        "/api/v1/preparation/claims",
        &json!({"claim_id": Uuid::new_v4()}),
    )
    .await
    .json();
    let cancel = format!(
        "/api/v1/inbox/lessons/{}/cancel",
        receipt["generation_id"].as_str().unwrap()
    );
    assert_eq!(
        post(&server.router, &cancel, &json!({})).await.status,
        StatusCode::OK
    );
    assert_eq!(
        post(&server.router, "/api/v1/preparation/renew", &fresh["fence"])
            .await
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        post(
            &server.router,
            "/api/v1/preparation/publish",
            &claim["fence"]
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
}
