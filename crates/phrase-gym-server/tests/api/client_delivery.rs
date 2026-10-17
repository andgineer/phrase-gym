use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::DateTime;
use phrase_gym_server::ServerSettings;

use crate::support::http::send;
use crate::support::{HOST, TestServer};

fn start(client_dir: Option<std::path::PathBuf>) -> TestServer {
    let settings = ServerSettings {
        client_dir,
        ..ServerSettings::loopback()
    }
    .with_hosts([HOST.to_owned()]);
    TestServer::with_settings(
        DateTime::parse_from_rfc3339("2026-10-12T12:00:00Z")
            .unwrap()
            .to_utc(),
        &settings,
    )
}

fn get(path: &str) -> Request<Body> {
    Request::get(path)
        .header("host", HOST)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn the_built_client_is_served_same_origin_without_caching_or_offline_assets() {
    let client = tempfile::tempdir().unwrap();
    std::fs::write(
        client.path().join("index.html"),
        "<main>Online client</main>",
    )
    .unwrap();
    std::fs::write(client.path().join("icon.svg"), "<svg></svg>").unwrap();
    std::fs::write(client.path().join("retire-worker.js"), "unregister()").unwrap();
    std::fs::create_dir_all(client.path().join("assets/fonts")).unwrap();
    std::fs::write(
        client.path().join("assets/index-Ab1_c.js"),
        "onlineClient()",
    )
    .unwrap();
    std::fs::write(client.path().join("assets/index-Ab1_c.css"), "main{}").unwrap();
    std::fs::write(client.path().join("assets/fonts/inter.woff2"), b"wOF2").unwrap();
    std::fs::write(client.path().join("assets/.hidden"), "secret").unwrap();
    let server = start(Some(client.path().to_owned()));

    for path in ["/", "/index.html", "/learn", "/learn/en/practice/retained"] {
        let reply = send(&server.router, get(path)).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(reply.body, "<main>Online client</main>");
        assert_eq!(
            reply.header("content-type"),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(reply.header("cache-control"), Some("no-store"));
    }
    for (path, kind, body) in [
        (
            "/assets/index-Ab1_c.js",
            "text/javascript; charset=utf-8",
            "onlineClient()",
        ),
        (
            "/assets/index-Ab1_c.css",
            "text/css; charset=utf-8",
            "main{}",
        ),
        ("/assets/fonts/inter.woff2", "font/woff2", "wOF2"),
        ("/icon.svg", "image/svg+xml", "<svg></svg>"),
    ] {
        let reply = send(&server.router, get(path)).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(reply.header("content-type"), Some(kind), "{path}");
        assert_eq!(reply.header("cache-control"), Some("no-store"), "{path}");
        assert_eq!(reply.body, body, "{path}");
    }
    for path in [
        "/sw.js",
        "/retire-worker.js",
        "/manifest.webmanifest",
        "/apple-touch-icon.png",
        "/assets/missing.js",
        "/assets/.hidden",
        "/assets/%2e%2e/index.html",
        "/assets/fonts/../../index.html",
        "/api/v1/unknown",
    ] {
        let reply = send(&server.router, get(path)).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(reply.header("cache-control"), Some("no-store"), "{path}");
    }
    let head = Request::head("/assets/index-Ab1_c.js")
        .header("host", HOST)
        .body(Body::empty())
        .unwrap();
    let reply = send(&server.router, head).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.body.is_empty());
}

#[tokio::test]
async fn a_missing_client_build_is_reported_instead_of_served() {
    let empty = tempfile::tempdir().unwrap();
    for client_dir in [None, Some(empty.path().to_owned())] {
        let server = start(client_dir);
        let reply = send(&server.router, get("/")).await;
        assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            reply.json()["detail"],
            "Build the client and pass --client-dir frontend/dist"
        );
        assert_eq!(
            send(&server.router, get("/learn/en")).await.status,
            StatusCode::NOT_FOUND
        );
    }
}
