use std::collections::BTreeSet;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use phrase_gym_core::clock::Clock;
use phrase_gym_core::hashing::sha256_hex;
use phrase_gym_core::timestamps;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::content::{WAV, get, package, post};
use crate::support::documents::base64;
use crate::support::http::{Reply, send};
use crate::support::practice::{
    apply, count, current, exposure, moment, rate, ready, session, started,
};
use crate::support::{HOST, TestServer, repository_root};

async fn presented() -> (TestServer, Value) {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    (server, result["session"].clone())
}

fn wav() -> Vec<u8> {
    std::fs::read(repository_root().join(WAV)).unwrap()
}

fn take(server: &TestServer, attempt: &Value, audio: &[u8], extra: &Value) -> Value {
    let mut body = json!({
        "id": Uuid::new_v4(), "target_id": attempt["target_id"], "revision": attempt["revision"],
        "attempt_id": attempt["id"], "captured_at": timestamps::isoformat(server.clock.now()),
        "mime_type": "audio/wav", "duration_ms": 100, "complete": true,
        "capture_settings": {"echoCancellation": true, "sampleRate": 48000, "deviceKind": "builtin"},
        "audio_base64": base64(audio),
    });
    for (key, value) in extra.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

async fn store(router: &axum::Router, body: &Value) -> Reply {
    post(router, "/api/v1/recordings", body).await
}

async fn ranged(router: &axum::Router, id: &Value, range: &str) -> Reply {
    let request = Request::get(format!("/api/v1/recordings/{}", id.as_str().unwrap()))
        .header("host", HOST)
        .header("range", range)
        .body(Body::empty())
        .unwrap();
    send(router, request).await
}

#[tokio::test]
async fn every_take_is_kept_idempotently_through_completion_and_restart() {
    let (server, state) = presented().await;
    let attempt = state["attempts"][0].clone();
    let bodies = [
        take(&server, &attempt, &wav(), &json!({})),
        take(
            &server,
            &attempt,
            b"interrupted",
            &json!({"complete": false}),
        ),
    ];
    let mut ids = Vec::new();
    for body in &bodies {
        let stored = store(&server.router, body).await;
        assert_eq!(stored.status, StatusCode::OK, "{:?}", stored.body);
        assert_eq!(store(&server.router, body).await.json(), stored.json());
        ids.push(stored.json()["id"].clone());
    }
    let mut changed = bodies[0].clone();
    changed["complete"] = json!(false);
    assert_eq!(
        store(&server.router, &changed).await.status,
        StatusCode::CONFLICT
    );
    let mut state = session(&server.router, &state["id"]).await;
    let (_, rated) = apply(&server, &state, "rate", &json!({"rating": "recalled"})).await;
    state = rated["session"].clone();
    while state["status"] == "active" {
        state = rate(&server, &state, "recalled").await["session"].clone();
    }
    let restarted = server.restarted();
    let target = attempt["target_id"].as_str().unwrap();
    let retained = get(&restarted, &format!("/api/v1/phrases/{target}/recordings"))
        .await
        .json();
    let retained_ids: BTreeSet<String> = retained
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].to_string())
        .collect();
    assert_eq!(retained_ids, ids.iter().map(Value::to_string).collect());
    let first = get(
        &restarted,
        &format!("/api/v1/recordings/{}", ids[0].as_str().unwrap()),
    )
    .await;
    assert_eq!(first.body.as_ref(), wav().as_slice());
    let second = get(
        &restarted,
        &format!("/api/v1/recordings/{}", ids[1].as_str().unwrap()),
    )
    .await;
    assert_eq!(second.body.as_ref(), b"interrupted");
}

#[tokio::test]
async fn a_lookup_take_needs_a_real_exposure_and_starts_no_session() {
    let (server, state) = presented().await;
    let phrase = &state["manifest"]["targets"][0]["phrase"];
    let exposure_id = exposure(&server.router, phrase, "explanation").await;
    let mut body = take(
        &server,
        &state["attempts"][0],
        &wav(),
        &json!({"exposure_id": exposure_id}),
    );
    body.as_object_mut().unwrap().remove("attempt_id");
    assert_eq!(store(&server.router, &body).await.status, StatusCode::OK);
    assert_eq!(current(&server.router).await["id"], state["id"]);
    body["id"] = json!(Uuid::new_v4());
    body["exposure_id"] = json!(Uuid::new_v4());
    assert_eq!(
        store(&server.router, &body).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn invalid_transfers_and_contexts_store_nothing() {
    let (server, state) = presented().await;
    let attempt = &state["attempts"][0];
    for (extra, status) in [
        (
            json!({"audio_base64": "not base64"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"audio_base64": ""}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (json!({"attempt_id": Uuid::new_v4()}), StatusCode::CONFLICT),
        (json!({"revision": 999}), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({"revision": 0}), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({"duration_ms": -1}), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({"mime_type": ""}), StatusCode::UNPROCESSABLE_ENTITY),
        (
            json!({"mime_type": "text/plain"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"captured_at": "2026-10-12T12:00:00"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"capture_settings": {"nested": {"no": 1}}}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"capture_event_id": Uuid::new_v4()}),
            StatusCode::CONFLICT,
        ),
        (json!({"source": "coach"}), StatusCode::UNPROCESSABLE_ENTITY),
    ] {
        let reply = store(&server.router, &take(&server, attempt, &wav(), &extra)).await;
        assert_eq!(reply.status, status, "{extra}: {:?}", reply.body);
    }
    let unknown = format!("/api/v1/recordings/{}", Uuid::new_v4());
    assert_eq!(
        get(&server.router, &unknown).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(count(&server, "learner_recording"), 0);
}

#[tokio::test]
async fn the_size_limit_is_twelve_mebibytes_of_decoded_audio() {
    let (server, state) = presented().await;
    let attempt = &state["attempts"][0];
    let largest = vec![9_u8; 12 * 1024 * 1024];
    let accepted = store(
        &server.router,
        &take(&server, attempt, &largest, &json!({})),
    )
    .await;
    assert_eq!(accepted.status, StatusCode::OK, "{:?}", accepted.body);
    assert_eq!(accepted.json()["byte_length"], 12 * 1024 * 1024);
    let oversized = vec![9_u8; 12 * 1024 * 1024 + 1];
    let refused = store(
        &server.router,
        &take(&server, attempt, &oversized, &json!({})),
    )
    .await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(count(&server, "learner_recording"), 1);
}

#[tokio::test]
async fn ranges_and_capture_support_are_pinned_to_the_take() {
    let (server, state) = presented().await;
    let recorded = json!({"modality": "recorded"});
    let (capture, result) = apply(&server, &state, "respond", &recorded).await;
    assert_eq!(result["receipts"][0]["capture_supported"], false);
    let attempt = result["session"]["attempts"][0].clone();
    let body = take(
        &server,
        &attempt,
        &wav(),
        &json!({"capture_event_id": capture["id"]}),
    );
    let first = store(&server.router, &body).await;
    assert_eq!(first.status, StatusCode::OK, "{:?}", first.body);
    let (_, result) = apply(&server, &result["session"], "capture_complete", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "respond", &recorded).await;
    assert_eq!(result["receipts"][0]["capture_supported"], true);
    assert_eq!(
        result["session"]["attempts"][0]["original"]["supported"],
        false
    );
    let id = &first.json()["id"];
    let length = wav().len();
    let partial = ranged(&server.router, id, "bytes=0-9").await;
    assert_eq!(partial.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(partial.body.as_ref(), &wav()[..10]);
    assert_eq!(
        partial.header("content-range"),
        Some(format!("bytes 0-9/{length}").as_str())
    );
    assert_eq!(
        ranged(&server.router, id, "bytes=-10").await.body.as_ref(),
        &wav()[length - 10..]
    );
    assert_eq!(
        ranged(&server.router, id, "bytes=999999-").await.status,
        StatusCode::RANGE_NOT_SATISFIABLE
    );
}

#[tokio::test]
async fn unfinished_captures_and_recovered_incomplete_takes_never_promote_a_schedule() {
    let (server, state) = presented().await;
    let recorded = json!({"modality": "recorded"});
    let (_, result) = apply(&server, &state, "respond", &recorded).await;
    assert_eq!(result["session"]["attempts"][0]["revealed"], false);
    let recalled = json!({"rating": "recalled"});
    let (_, result) = apply(&server, &result["session"], "rate", &recalled).await;
    assert_eq!(
        result["receipts"][0]["eligibility_reason"],
        "technical_failure"
    );
    assert_eq!(result["receipts"][0]["schedule_changed"], false);
    let (_, result) = apply(&server, &result["session"], "present", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "respond", &recorded).await;
    let body = take(
        &server,
        &result["session"]["attempts"][1],
        &wav(),
        &json!({"complete": false}),
    );
    assert_eq!(store(&server.router, &body).await.status, StatusCode::OK);
    let current = session(&server.router, &state["id"]).await;
    let (_, result) = apply(&server, &current, "capture_complete", &json!({})).await;
    let (_, result) = apply(&server, &result["session"], "rate", &recalled).await;
    assert_eq!(
        result["receipts"][0]["eligibility_reason"],
        "technical_failure"
    );
    assert_eq!(result["receipts"][0]["schedule_changed"], false);
}

#[tokio::test]
async fn encoded_and_opaque_takes_keep_their_original_bytes() {
    let (server, state) = presented().await;
    let (capture, result) =
        apply(&server, &state, "respond", &json!({"modality": "recorded"})).await;
    let attempt = result["session"]["attempts"][0].clone();
    let originals: Vec<(Vec<u8>, &str)> = vec![
        (
            std::fs::read(repository_root().join("fixtures/preservation/media/tone.mp3")).unwrap(),
            "audio/mpeg",
        ),
        (
            std::fs::read(repository_root().join("fixtures/preservation/media/tone.ogg")).unwrap(),
            "audio/ogg",
        ),
        (
            std::fs::read(repository_root().join("fixtures/preservation/media/tone.m4a")).unwrap(),
            "audio/mp4",
        ),
        (
            b"opaque-browser-audio\x00\xff".to_vec(),
            "audio/mp4; codecs=mp4a.40.2",
        ),
        (
            b"unfinished-container-fragment".to_vec(),
            "AUDIO/WEBM;codecs=opus",
        ),
    ];
    for (audio, mime) in &originals {
        let extra = json!({"mime_type": mime, "capture_event_id": capture["id"]});
        let body = take(&server, &attempt, audio, &extra);
        let reply = store(&server.router, &body).await;
        assert_eq!(reply.status, StatusCode::OK, "{mime}: {:?}", reply.body);
        let receipt = reply.json();
        assert_eq!(receipt["byte_length"], audio.len());
        assert_eq!(receipt["sha256"], sha256_hex(audio));
        assert_eq!(store(&server.router, &body).await.json(), receipt);
        let played = get(
            &server.router,
            &format!("/api/v1/recordings/{}", receipt["id"].as_str().unwrap()),
        )
        .await;
        assert_eq!(played.body.as_ref(), audio.as_slice());
        assert_eq!(played.header("content-type"), Some(*mime));
        let mut relabelled = body.clone();
        relabelled["mime_type"] = json!("audio/wav");
        assert_eq!(
            store(&server.router, &relabelled).await.status,
            StatusCode::CONFLICT
        );
    }
    let unchanged = session(&server.router, &state["id"]).await;
    assert_eq!(unchanged, result["session"]);
    assert_eq!(unchanged["attempts"][0]["technical_failure"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_upload_retries_store_one_take_with_one_receipt() {
    let (server, state) = presented().await;
    let body = take(&server, &state["attempts"][0], &wav(), &json!({}));
    let tasks: Vec<_> = (0..6)
        .map(|_| {
            let (router, body) = (server.router.clone(), body.clone());
            tokio::spawn(async move { store(&router, &body).await })
        })
        .collect();
    let mut receipts = Vec::new();
    for task in tasks {
        let reply = task.await.unwrap();
        assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
        receipts.push(reply.json());
    }
    assert!(receipts.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(count(&server, "learner_recording"), 1);
}
