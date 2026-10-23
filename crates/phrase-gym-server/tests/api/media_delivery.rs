use axum::body::Body;
use axum::http::{Request, StatusCode};
use phrase_gym_core::clock::Clock;
use phrase_gym_core::hashing::sha256_hex;
use phrase_gym_core::timestamps;
use serde_json::json;
use uuid::Uuid;

use crate::support::content::{WAV, package, post};
use crate::support::documents::base64;
use crate::support::http::{Reply, send};
use crate::support::practice::{apply, moment, ready, started};
use crate::support::{HOST, TestServer, repository_root};

async fn fetch(server: &TestServer, path: &str, range: Option<&str>) -> Reply {
    let mut request = Request::get(path).header("host", HOST);
    if let Some(range) = range {
        request = request.header("range", range);
    }
    send(&server.router, request.body(Body::empty()).unwrap()).await
}

fn assert_original_headers(reply: &Reply, mime: &str, sha256: &str, length: usize) {
    assert_eq!(reply.header("content-type"), Some(mime));
    assert_eq!(reply.header("accept-ranges"), Some("bytes"));
    assert_eq!(reply.header("etag"), Some(format!("\"{sha256}\"").as_str()));
    assert_eq!(reply.header("cache-control"), Some("no-store"));
    assert_eq!(
        reply.header("content-length"),
        Some(reply.body.len().to_string().as_str())
    );
    assert!(reply.body.len() <= length);
}

// Media elements probe with a two-byte range, then stream an open-ended tail; both paths share it.
#[tokio::test]
async fn takes_and_references_deliver_exact_original_bytes_and_ranges() {
    let server = TestServer::start(moment("2026-10-12T12:00:00+00:00"));
    let mut content = package();
    ready(&server.router, &mut content).await;
    let state = started(&server.router, &json!({})).await;
    let (_, result) = apply(&server, &state, "present", &json!({})).await;
    let attempt = &result["session"]["attempts"][0];
    let take =
        std::fs::read(repository_root().join("fixtures/preservation/media/tone.m4a")).unwrap();
    let stored = post(
        &server.router,
        "/api/v1/recordings",
        &json!({
            "id": Uuid::new_v4(), "target_id": attempt["target_id"], "revision": attempt["revision"],
            "attempt_id": attempt["id"], "captured_at": timestamps::isoformat(server.clock.now()),
            "mime_type": "audio/mp4", "duration_ms": 1000, "complete": true,
            "capture_settings": {}, "audio_base64": base64(&take),
        }),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{:?}", stored.body);
    let reference = std::fs::read(repository_root().join(WAV)).unwrap();
    for (path, bytes, mime) in [
        (
            format!(
                "/api/v1/recordings/{}",
                stored.json()["id"].as_str().unwrap()
            ),
            take,
            "audio/mp4",
        ),
        (
            format!(
                "/api/v1/media/{}",
                content["targets"][0]["reference_audio_id"]
                    .as_str()
                    .unwrap()
            ),
            reference,
            "audio/wav",
        ),
    ] {
        let (length, sha256) = (bytes.len(), sha256_hex(&bytes));
        let whole = fetch(&server, &path, None).await;
        assert_eq!(whole.status, StatusCode::OK, "{path}");
        assert_original_headers(&whole, mime, &sha256, length);
        assert_eq!(whole.body.as_ref(), bytes.as_slice());
        let middle = length / 2;
        for (range, start, end) in [
            ("bytes=0-1".to_owned(), 0, 1),
            (format!("bytes={middle}-"), middle, length - 1),
            (format!("bytes=-{}", length - middle), middle, length - 1),
        ] {
            let part = fetch(&server, &path, Some(&range)).await;
            assert_eq!(part.status, StatusCode::PARTIAL_CONTENT, "{path} {range}");
            assert_original_headers(&part, mime, &sha256, length);
            assert_eq!(
                part.header("content-range"),
                Some(format!("bytes {start}-{end}/{length}").as_str())
            );
            assert_eq!(part.body.as_ref(), &bytes[start..=end]);
        }
        let beyond = fetch(&server, &path, Some(&format!("bytes={length}-"))).await;
        assert_eq!(beyond.status, StatusCode::RANGE_NOT_SATISFIABLE);
        assert_eq!(
            beyond.header("content-range"),
            Some(format!("bytes */{length}").as_str())
        );
    }
}
