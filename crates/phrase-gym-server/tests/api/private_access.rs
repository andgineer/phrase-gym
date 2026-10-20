use crate::support::TestServer;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use tower::ServiceExt;

#[tokio::test]
async fn local_operator_needs_no_application_credentials_or_cookie() {
    let server = TestServer::start(Utc::now());
    for path in ["/api/v1/settings", "/api/v1/profile"] {
        let request = Request::get(path)
            .header("host", "localhost")
            .extension(ConnectInfo(
                "127.0.0.1:1234".parse::<std::net::SocketAddr>().unwrap(),
            ))
            .body(Body::empty())
            .unwrap();
        let response = server.router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.headers().contains_key("set-cookie"));
    }
}

#[tokio::test]
async fn remote_peers_cannot_spoof_a_trusted_network_connection() {
    let server = TestServer::start(Utc::now());
    for peer in [
        None,
        Some("192.168.1.50:1234"),
        Some("100.100.1.2:1234"),
        Some("203.0.113.1:1234"),
    ] {
        let mut builder = Request::get("/api/v1/settings")
            .header("host", "localhost")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-forwarded-proto", "https");
        if let Some(peer) = peer {
            builder = builder.extension(ConnectInfo(peer.parse::<std::net::SocketAddr>().unwrap()));
        }
        let response = server
            .router
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn remote_hosts_are_rejected_even_from_a_local_proxy() {
    let server = TestServer::start(Utc::now());
    for host in ["gym.example.ts.net", "public.example", "192.168.1.2"] {
        let request = Request::get("/api/v1/settings")
            .header("host", host)
            .extension(ConnectInfo(
                "127.0.0.1:1234".parse::<std::net::SocketAddr>().unwrap(),
            ))
            .body(Body::empty())
            .unwrap();
        let response = server.router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
