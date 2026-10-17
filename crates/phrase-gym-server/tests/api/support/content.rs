use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};

use super::documents::base64;
use super::http::{Reply, send};
use super::{HOST, repository_root};

pub fn package() -> Value {
    serde_json::from_str(&package_text()).unwrap()
}

pub async fn get(router: &Router, path: &str) -> Reply {
    let request = Request::get(path)
        .header("host", HOST)
        .body(Body::empty())
        .unwrap();
    send(router, request).await
}

pub fn package_text() -> String {
    std::fs::read_to_string(repository_root().join("fixtures/preservation/packages/en-basic.json"))
        .unwrap()
}

pub async fn post_text(router: &Router, path: &str, body: String) -> Reply {
    let request = Request::post(path)
        .header("host", HOST)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    send(router, request).await
}

pub async fn post(router: &Router, path: &str, body: &Value) -> Reply {
    post_text(router, path, serde_json::to_string(body).unwrap()).await
}

pub async fn stage(router: &Router, package: &Value) -> Reply {
    post(router, "/api/v1/imports", package).await
}

pub async fn publish(router: &Router, package: &Value) -> Value {
    let staged = stage(router, package).await;
    assert_eq!(staged.status, StatusCode::OK, "{:?}", staged.body);
    let report = staged.json();
    assert_eq!(report["status"], "staged", "{report}");
    let published = post(
        router,
        &format!("/api/v1/imports/{}/publish", report["id"].as_str().unwrap()),
        &json!({}),
    )
    .await;
    assert_eq!(published.status, StatusCode::OK, "{:?}", published.body);
    published.json()
}

pub async fn phrases(router: &Router, language: &str, query: &str) -> Vec<Value> {
    let reply = get(
        router,
        &format!("/api/v1/phrases/search?language={language}&q={query}"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    reply.json()["items"].as_array().unwrap().clone()
}

pub async fn upload(router: &Router, metadata: &Value, file: &str, mime: &str) -> Value {
    let mut provenance = json!({
        "language": "en",
        "text": "How has your day been so far?",
        "role": "reference",
        "origin": "actor",
        "source": "acceptance-test",
        "performer": "Synthetic test fixture",
        "locale": "en-US",
        "preparation_version": "1",
    });
    for (key, value) in metadata.as_object().unwrap() {
        provenance[key] = value.clone();
    }
    let bytes = std::fs::read(repository_root().join(file)).unwrap();
    let reply = post(
        router,
        "/api/v1/media",
        &json!({"metadata": provenance, "mime_type": mime, "audio_base64": base64(&bytes)}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    reply.json()
}

pub const WAV: &str = "fixtures/preservation/media/silence-100ms.wav";
