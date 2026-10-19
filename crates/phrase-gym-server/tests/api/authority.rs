use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::profiles::Profiles;
use phrase_gym_core::storage::Database;
use phrase_gym_server::{AppState, Authority, Capability, Credential, ServerSettings, router};
use tower::ServiceExt;

fn token(actor: &str) -> String {
    format!("synthetic-{actor}-000000000000000000000000")
}

fn app(directory: &std::path::Path) -> axum::Router {
    let database = Arc::new(Database::open(directory).unwrap());
    Profiles::new(database.clone(), Arc::new(SystemClock))
        .initialize_current_learner()
        .unwrap();
    let credentials = [
        ("learner", vec![Capability::Learner]),
        ("material", vec![Capability::MaterialSubmit]),
        ("worker", vec![Capability::Worker]),
        ("reader", vec![Capability::ProfileRead]),
        (
            "editor",
            vec![Capability::ProfileRead, Capability::ProfileUpdate],
        ),
    ]
    .into_iter()
    .map(|(actor, capabilities)| Credential {
        actor: actor.into(),
        token: token(actor),
        capabilities,
    })
    .collect();
    let settings = ServerSettings {
        authority: Arc::new(Authority::new(credentials).unwrap()),
        ..ServerSettings::loopback()
    };
    router(
        AppState::new(
            database,
            Arc::new(MediaStore::open(directory).unwrap()),
            Arc::new(SystemClock),
        ),
        &settings,
    )
}

async fn status(app: &axum::Router, actor: Option<&str>, method: &str, path: &str) -> StatusCode {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .header("content-type", "application/json");
    if let Some(actor) = actor {
        request = request.header("authorization", format!("Bearer {}", token(actor)));
    }
    app.clone()
        .oneshot(request.body(Body::from("{}")).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn capabilities_authenticate_before_parsing_and_isolate_takes_profiles_and_material() {
    let directory = tempfile::tempdir().unwrap();
    let app = app(directory.path());
    let take = format!("/api/v1/recordings/{}", uuid::Uuid::new_v4());
    for path in ["/api/v1/settings", "/api/v1/profile", &take] {
        assert_eq!(
            status(&app, None, "GET", path).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(&app, Some("unknown"), "GET", path).await,
            StatusCode::UNAUTHORIZED
        );
    }
    for actor in ["material", "worker", "reader", "editor"] {
        assert_eq!(
            status(&app, Some(actor), "GET", &take).await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            status(&app, Some(actor), "POST", "/api/v1/recordings").await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            status(&app, Some(actor), "GET", "/api/v1/lessons?language=en").await,
            StatusCode::FORBIDDEN
        );
    }
    for actor in ["material", "worker", "learner", "reader"] {
        assert_eq!(
            status(&app, Some(actor), "PUT", "/api/v1/profile").await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            status(&app, Some(actor), "POST", "/api/v1/profile/restore").await,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        status(&app, Some("editor"), "PUT", "/api/v1/profile").await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for actor in ["worker", "reader", "editor", "learner"] {
        assert_eq!(
            status(&app, Some(actor), "GET", "/api/v1/profile").await,
            StatusCode::OK
        );
    }
    assert_eq!(
        status(&app, Some("material"), "GET", "/api/v1/profile").await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(&app, Some("learner"), "POST", "/api/v1/media").await,
        StatusCode::FORBIDDEN
    );
    for actor in ["material", "worker"] {
        assert_eq!(
            status(&app, Some(actor), "POST", "/api/v1/media").await,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let request = Request::get("/api/v1/settings")
        .header("host", "localhost")
        .header("x-user", "learner")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(request).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn learner_login_uses_http_only_cookie_and_cannot_authorize_other_capabilities() {
    let directory = tempfile::tempdir().unwrap();
    let app = app(directory.path());
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/auth/session")
                .header("host", "localhost")
                .header("authorization", format!("Bearer {}", token("learner")))
                .header("origin", "http://localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let cookie = response.headers()["set-cookie"].to_str().unwrap();
    assert!(cookie.contains("HttpOnly; SameSite=Strict"));
    let cookie = cookie.split(';').next().unwrap();
    for (method, path, expected) in [
        ("GET", "/api/v1/settings", StatusCode::OK),
        ("PUT", "/api/v1/profile", StatusCode::FORBIDDEN),
        ("POST", "/api/v1/media", StatusCode::FORBIDDEN),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("host", "localhost")
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/profile")
                .header("host", "localhost")
                .header("cookie", format!("phrase_gym_learner={}", token("editor")))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        status(&app, Some("worker"), "POST", "/api/v1/auth/session").await,
        StatusCode::FORBIDDEN
    );
    let response = app
        .oneshot(
            Request::get("/login")
                .header("host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let html = response.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&html).contains("Learner access token"));
}

#[tokio::test]
async fn unauthorized_uploads_are_rejected_before_consuming_the_body() {
    let directory = tempfile::tempdir().unwrap();
    let app = app(directory.path());
    for (path, token, expected) in [
        ("/api/v1/media", None, StatusCode::UNAUTHORIZED),
        (
            "/api/v1/profile",
            Some(token("worker")),
            StatusCode::FORBIDDEN,
        ),
    ] {
        let body = crate::support::http::Chunks(std::collections::VecDeque::from([Err(
            std::io::Error::other("This body must not be consumed"),
        )]));
        let mut request = Request::builder()
            .method(if path.ends_with("profile") {
                "PUT"
            } else {
                "POST"
            })
            .uri(path)
            .header("host", "localhost");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        assert_eq!(
            app.clone()
                .oneshot(request.body(Body::new(body)).unwrap())
                .await
                .unwrap()
                .status(),
            expected
        );
    }
}
