use crate::support::TestServer;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use phrase_gym_server::ServerSettings;
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
    let settings = ServerSettings::loopback()
        .with_tailscale_origin("https://gym.example.ts.net")
        .unwrap();
    let server = TestServer::with_settings(Utc::now(), &settings);
    for peer in [
        None,
        Some("192.168.1.50:1234"),
        Some("100.100.1.2:1234"),
        Some("203.0.113.1:1234"),
    ] {
        let mut builder = Request::get("/api/v1/settings")
            .header("host", "gym.example.ts.net")
            .header("tailscale-user-login", "forged@example.com")
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
async fn explicit_serve_proxy_accepts_tagged_nodes_and_rejects_funnel() {
    let settings = ServerSettings::loopback()
        .with_tailscale_origin("https://gym.example.ts.net")
        .unwrap();
    let server = TestServer::with_settings(Utc::now(), &settings);
    for (funnel, expected) in [(false, StatusCode::OK), (true, StatusCode::FORBIDDEN)] {
        let mut builder = Request::get("/api/v1/settings")
            .header("host", "gym.example.ts.net")
            .extension(ConnectInfo(
                "127.0.0.1:1234".parse::<std::net::SocketAddr>().unwrap(),
            ));
        if funnel {
            builder = builder
                .header("tailscale-funnel-request", "?1")
                .header("tailscale-user-login", "forged@example.com");
        }
        let response = server
            .router
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
}

#[test]
fn serve_configuration_rejects_public_origins_wildcards_and_user_information() {
    for origin in [
        "http://gym.example.ts.net",
        "https://public.example",
        "https://*.ts.net",
        "https://user@gym.example.ts.net",
        "https://gym.example.ts.net/path",
        "https://.ts.net",
    ] {
        assert!(
            ServerSettings::loopback()
                .with_tailscale_origin(origin)
                .is_err(),
            "{origin}"
        );
    }
}
