use std::collections::VecDeque;

use axum::body::{Body, Bytes};
use axum::http::{Request, StatusCode};
use chrono::DateTime;
use phrase_gym_server::ServerSettings;

use crate::support::http::{Chunks, send};
use crate::support::{HOST, TestServer};

const LIMIT: usize = 24 * 1024 * 1024;
const TARGET: &str = "2f0c6f1e-5a43-4c1b-9b7e-3d2a1c0e9f10";

fn server() -> TestServer {
    let settings = ServerSettings {
        allowed_origins: vec!["https://phrase-gym.example".to_owned()],
        ..ServerSettings::loopback()
    }
    .with_hosts([HOST.to_owned(), "*.tailnet.example".to_owned()]);
    let server = TestServer::with_settings(
        DateTime::parse_from_rfc3339("2026-10-12T12:00:00Z")
            .unwrap()
            .to_utc(),
        &settings,
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

fn exposure() -> String {
    format!(
        r#"{{"id":"6a1f3c1e-0d7b-4e43-9a51-3f3e2f6b8a01","target_id":"{TARGET}","revision":1,"language":"en","kind":"answer"}}"#
    )
}

fn post(headers: &[(&str, &str)], body: Body) -> Request<Body> {
    let mut builder = Request::post("/api/v1/exposures").header("content-type", "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    builder.body(body).unwrap()
}

fn stored(server: &TestServer) -> i64 {
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .query_row("SELECT count(*) FROM exposure", [], |row| row.get(0))
        .unwrap()
}

#[tokio::test]
async fn cross_site_mutations_are_rejected_and_same_or_allowed_origins_pass() {
    let server = server();
    for headers in [
        vec![("host", HOST), ("origin", "https://untrusted.example")],
        vec![("host", HOST), ("origin", "https://testserver")],
        vec![("host", HOST), ("origin", "null")],
        vec![("host", HOST), ("sec-fetch-site", "cross-site")],
    ] {
        let reply = send(&server.router, post(&headers, Body::from(exposure()))).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{headers:?}");
        assert_eq!(
            reply.json()["detail"],
            "Cross-origin mutations are not allowed"
        );
        assert_eq!(reply.header("cache-control"), Some("no-store"));
    }
    assert_eq!(stored(&server), 0);
    for headers in [
        vec![("host", HOST), ("origin", "http://testserver")],
        vec![("host", HOST), ("origin", "https://phrase-gym.example")],
        vec![("host", HOST), ("sec-fetch-site", "same-origin")],
    ] {
        let reply = send(&server.router, post(&headers, Body::from(exposure()))).await;
        assert_eq!(reply.status, StatusCode::OK, "{headers:?}");
    }
    let read = Request::get("/api/v1/unknown")
        .header("host", HOST)
        .header("origin", "https://untrusted.example")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        send(&server.router, read).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn untrusted_or_missing_hosts_are_rejected_for_every_method() {
    let server = server();
    let get = |host: Option<&str>| {
        let mut builder = Request::get("/api/v1/unknown");
        if let Some(host) = host {
            builder = builder.header("host", host);
        }
        builder.body(Body::empty()).unwrap()
    };
    for host in [Some("untrusted.example"), Some("tailnet.example"), None] {
        let reply = send(&server.router, get(host)).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{host:?}");
        assert_eq!(reply.body, "Invalid host header");
        assert_eq!(reply.header("cache-control"), Some("no-store"));
    }
    for host in ["testserver:8000", "localhost", "phone.tailnet.example"] {
        assert_eq!(
            send(&server.router, get(Some(host))).await.status,
            StatusCode::NOT_FOUND,
            "{host}"
        );
    }
    let mutation = post(&[("host", "untrusted.example")], Body::from(exposure()));
    assert_eq!(
        send(&server.router, mutation).await.status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn declared_oversized_or_unparsable_lengths_are_rejected_before_reading() {
    let server = server();
    for length in [(LIMIT + 1).to_string(), "invalid".to_owned()] {
        let request = post(
            &[("host", HOST), ("content-length", &length)],
            Body::from(exposure()),
        );
        let reply = send(&server.router, request).await;
        assert_eq!(reply.status, StatusCode::PAYLOAD_TOO_LARGE, "{length}");
        assert_eq!(reply.json()["detail"], "Request exceeds 24 MiB");
    }
    assert_eq!(stored(&server), 0);
}

#[tokio::test]
async fn the_actual_stream_is_bounded_whatever_the_declared_length() {
    let server = server();
    for declared in [None, Some("2")] {
        let chunks = Chunks(VecDeque::from([
            Ok(Bytes::from(vec![b' '; LIMIT / 2])),
            Ok(Bytes::from(vec![b' '; LIMIT / 2 + 1])),
        ]));
        let mut headers = vec![("host", HOST)];
        if let Some(length) = declared {
            headers.push(("content-length", length));
        }
        let reply = send(&server.router, post(&headers, Body::new(chunks))).await;
        assert_eq!(reply.status, StatusCode::PAYLOAD_TOO_LARGE, "{declared:?}");
    }
    assert_eq!(stored(&server), 0);
}

#[tokio::test]
async fn split_bodies_reach_the_handler_once_and_disconnects_never_do() {
    let server = server();
    let text = exposure();
    let (head, tail) = text.split_at(20);
    let split = Chunks(VecDeque::from([
        Ok(Bytes::from(head.to_owned())),
        Ok(Bytes::from(tail.to_owned())),
    ]));
    let reply = send(&server.router, post(&[("host", HOST)], Body::new(split))).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(stored(&server), 1);

    let other = text.replace("6a1f3c1e", "7b2f3c1e");
    let disconnected = Chunks(VecDeque::from([
        Ok(Bytes::from(other.into_bytes())),
        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "client went away",
        )),
    ]));
    let reply = send(
        &server.router,
        post(&[("host", HOST)], Body::new(disconnected)),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(stored(&server), 1);
}
