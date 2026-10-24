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

#[tokio::test]
async fn explicit_serve_proxy_accepts_only_its_origin_and_never_funnel() {
    let settings = ServerSettings::loopback()
        .with_tailscale_origin("https://gym.example.ts.net:8443")
        .unwrap();
    let server = TestServer::with_settings(Utc::now(), &settings);
    for (funnel, origin, allowed) in [
        (false, "https://gym.example.ts.net:8443", true),
        (true, "https://gym.example.ts.net:8443", false),
        (false, "https://other.example.ts.net", false),
    ] {
        let mut builder = Request::post("/api/v1/sessions")
            .header("host", "gym.example.ts.net:8443")
            .header("origin", origin)
            .header("content-type", "application/json")
            .extension(ConnectInfo(
                "127.0.0.1:1234".parse::<std::net::SocketAddr>().unwrap(),
            ));
        if funnel {
            builder = builder.header("tailscale-funnel-request", "?1");
        }
        let response = server
            .router
            .clone()
            .oneshot(builder.body(Body::from("{}")).unwrap())
            .await
            .unwrap();
        let status = response.status();
        assert_eq!(
            status != StatusCode::FORBIDDEN,
            allowed,
            "{origin} funnel={funnel}"
        );
        assert_ne!(status, StatusCode::BAD_REQUEST, "{origin} funnel={funnel}");
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
        "https://gym.example.ts.net:http",
        "https://gym.example.ts.net:0443",
        "https://gym.example.ts.net:443",
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
