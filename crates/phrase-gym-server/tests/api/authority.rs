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

#[tokio::test]
async fn inbox_and_jobs_have_separate_capabilities() {
    let directory = tempfile::tempdir().unwrap();
    let app = app(directory.path());
    for actor in ["learner", "worker", "reader", "editor"] {
        assert_eq!(
            status(&app, Some(actor), "POST", "/api/v1/inbox/lessons").await,
            StatusCode::FORBIDDEN
        );
    }
    for actor in ["learner", "material", "reader", "editor"] {
        for path in [
            "claims", "stage", "publish", "receipt", "renew", "fail", "audio",
        ] {
            assert_eq!(
                status(
                    &app,
                    Some(actor),
                    "POST",
                    &format!("/api/v1/preparation/{path}")
                )
                .await,
                StatusCode::FORBIDDEN
            );
        }
    }
    assert_eq!(
        status(&app, Some("material"), "POST", "/api/v1/inbox/lessons").await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        status(&app, Some("worker"), "POST", "/api/v1/preparation/claims").await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn inbox_status_is_learner_readable_but_submission_and_status_remain_scoped() {
    let directory = tempfile::tempdir().unwrap();
    let app = app(directory.path());
    let id = uuid::Uuid::new_v4();
    let receipt = serde_json::json!({
        "generation_id": id, "submission_id": uuid::Uuid::new_v4(), "generation": 1,
        "namespace": "synthetic", "profile_revision": 1, "guide_version": "english-preview-v1",
        "received_at": "2026-10-19T10:00:00Z"
    });
    let database = Database::open(directory.path()).unwrap();
    rusqlite::Connection::open(database.path()).unwrap().execute(
        "INSERT INTO inbox_generation (id, actor, item_id, generation, submission_id, request, receipt, profile_revision, guide_version, status) VALUES (?1, 'learner', 'synthetic', 1, ?2, '{}', ?3, 1, 'english-preview-v1', 'waiting')",
        rusqlite::params![id.to_string(), receipt["submission_id"].as_str().unwrap(), receipt.to_string()],
    ).unwrap();
    let path = format!("/api/v1/inbox/lessons/{id}");
    assert_eq!(
        status(&app, None, "GET", &path).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status(&app, Some("learner"), "GET", &path).await,
        StatusCode::OK
    );
    assert_eq!(
        status(&app, Some("material"), "GET", &path).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status(&app, Some("worker"), "GET", &path).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(&app, Some("learner"), "POST", "/api/v1/inbox/lessons").await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn submission_receipts_are_read_only_scoped_and_require_learner_or_submit_authority() {
    let directory = tempfile::tempdir().unwrap();
    let app = app(directory.path());
    let submission = uuid::Uuid::new_v4();
    let receipt = serde_json::json!({"generation_id": uuid::Uuid::new_v4(), "submission_id": submission,
        "generation": 1, "namespace": "synthetic", "profile_revision": 1, "guide_version": "english-preview-v1",
        "received_at": "2026-10-19T10:00:00Z"});
    let database = Database::open(directory.path()).unwrap();
    rusqlite::Connection::open(database.path()).unwrap().execute(
        "INSERT INTO inbox_generation (id, actor, item_id, generation, submission_id, request, receipt, profile_revision, guide_version, status) VALUES (?1, 'learner', 'synthetic', 1, ?2, '{}', ?3, 1, 'english-preview-v1', 'published')",
        rusqlite::params![receipt["generation_id"].as_str().unwrap(), submission.to_string(), receipt.to_string()],
    ).unwrap();
    let path = format!("/api/v1/inbox/submissions/{submission}");
    assert_eq!(
        status(&app, None, "GET", &path).await,
        StatusCode::UNAUTHORIZED
    );
    for actor in ["worker", "reader", "editor"] {
        assert_eq!(
            status(&app, Some(actor), "GET", &path).await,
            StatusCode::FORBIDDEN
        );
    }
    for (actor, expected) in [("learner", receipt), ("material", serde_json::Value::Null)] {
        let request = Request::get(&path)
            .header("host", "localhost")
            .header("authorization", format!("Bearer {}", token(actor)))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            expected
        );
    }
}
