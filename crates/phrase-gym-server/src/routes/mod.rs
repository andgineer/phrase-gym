pub(crate) mod exposures;
pub(crate) mod inbox;
pub(crate) mod jobs;
pub(crate) mod library;
pub(crate) mod media;
mod params;
pub(crate) mod practice;
pub(crate) mod profiles;
pub(crate) mod readiness;
pub(crate) mod recordings;
#[cfg(feature = "test-support")]
pub(crate) mod seed;
pub(crate) mod sessions;
pub(crate) mod settings;

use axum::Router;
use axum::routing::{get, post};

use crate::authority::{Authority, Capability, Grant};
use crate::state::AppState;
use axum::middleware;
use std::sync::Arc;

pub(crate) fn api(authority: &Arc<Authority>) -> Router<AppState> {
    let protect = |any| {
        middleware::from_fn_with_state(
            Grant {
                authority: authority.clone(),
                any,
            },
            crate::authority::enforce,
        )
    };
    let routes = Router::new()
        .merge(inbox::routes().route_layer(protect(vec![Capability::MaterialSubmit])))
        .merge(
            Router::new()
                .route("/api/v1/inbox/lessons/{id}", get(inbox::status))
                .route_layer(protect(vec![
                    Capability::Learner,
                    Capability::MaterialSubmit,
                ])),
        )
        .merge(jobs::routes().route_layer(protect(vec![Capability::Worker])))
        .merge(learner().route_layer(protect(vec![Capability::Learner])))
        .merge(material().route_layer(protect(vec![
            Capability::MaterialSubmit,
            Capability::Worker,
        ])))
        .merge(
            Router::new()
                .route("/api/v1/profile", get(profiles::current))
                .route(
                    "/api/v1/profile/revisions/{revision}",
                    get(profiles::revision),
                )
                .route_layer(protect(vec![
                    Capability::ProfileRead,
                    Capability::Worker,
                    Capability::Learner,
                ])),
        )
        .merge(
            Router::new()
                .route("/api/v1/profile", axum::routing::put(profiles::update))
                .route("/api/v1/profile/restore", post(profiles::restore))
                .route_layer(protect(vec![Capability::ProfileUpdate])),
        );
    #[cfg(feature = "test-support")]
    let routes =
        routes.merge(seed::routes().route_layer(protect(vec![Capability::MaterialSubmit])));
    routes
}

// What the practising learner's browser uses, including learner recordings. Reference-media
// publication belongs to material routes, which Coach and external material credentials will reach;
// learner recordings must never be added there.
fn learner() -> Router<AppState> {
    Router::new()
        .route("/api/v1/media/reconcile", post(media::reconcile))
        .route("/api/v1/exposures", post(exposures::expose))
        .route("/api/v1/lessons", get(library::lessons))
        .route(
            "/api/v1/lessons/{lesson_id}/phrases",
            get(library::lesson_phrases),
        )
        .route("/api/v1/phrases/search", get(library::search))
        .route("/api/v1/phrases/{target_id}", get(library::phrase))
        .route(
            "/api/v1/phrases/{target_id}/recordings",
            get(recordings::list),
        )
        .route("/api/v1/dialogues", get(library::dialogues))
        .route("/api/v1/dialogues/{dialogue_id}", get(library::dialogue))
        .route("/api/v1/media/{media_id}", get(media::play))
        .route("/api/v1/recordings", post(recordings::store))
        .route("/api/v1/recordings/{recording_id}", get(recordings::play))
        .route("/api/v1/sessions", post(sessions::start))
        .route("/api/v1/sessions/current", get(sessions::current))
        .route("/api/v1/sessions/readiness", get(readiness::readiness))
        .route(
            "/api/v1/sessions/{session_id}",
            get(sessions::read).patch(sessions::transition),
        )
        .route(
            "/api/v1/sessions/{session_id}/events",
            get(practice::receipts).post(practice::submit),
        )
        .route(
            "/api/v1/settings",
            get(settings::read).put(settings::replace),
        )
}

fn material() -> Router<AppState> {
    Router::new()
        .route("/api/v1/media", post(media::upload))
        .route("/api/v1/media/{media_id}/metadata", get(media::metadata))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use phrase_gym_core::clock::SystemClock;
    use phrase_gym_core::media::store::MediaStore;
    use phrase_gym_core::storage::Database;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn learner_recordings_are_reachable_only_through_learner_routes() {
        let directory = tempfile::tempdir().unwrap();
        let state = AppState::new(
            Arc::new(Database::open(directory.path()).unwrap()),
            Arc::new(MediaStore::open(directory.path()).unwrap()),
            Arc::new(SystemClock),
        );
        let take = format!("/api/v1/recordings/{}", uuid::Uuid::new_v4());
        let list = format!("/api/v1/phrases/{}/recordings", uuid::Uuid::new_v4());
        for (method, path) in [
            ("GET", take.as_str()),
            ("POST", "/api/v1/recordings"),
            ("GET", list.as_str()),
        ] {
            let request = || {
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap()
            };
            let material = material()
                .with_state(state.clone())
                .oneshot(request())
                .await;
            let material = material.unwrap();
            assert_eq!(material.status(), StatusCode::NOT_FOUND, "{method} {path}");
            let unrouted = material.into_body().collect().await.unwrap().to_bytes();
            assert!(unrouted.is_empty(), "{method} {path}");
            let learner = learner().with_state(state.clone()).oneshot(request()).await;
            let handled = learner
                .unwrap()
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes();
            assert!(!handled.is_empty(), "{method} {path}");
        }
    }
}
