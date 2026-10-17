pub(crate) mod exposures;
pub(crate) mod library;
pub(crate) mod media;
mod params;
pub(crate) mod readiness;
#[cfg(feature = "test-support")]
pub(crate) mod seed;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub(crate) fn api() -> Router<AppState> {
    let routes = Router::new()
        .route("/api/v1/exposures", post(exposures::expose))
        .route("/api/v1/lessons", get(library::lessons))
        .route(
            "/api/v1/lessons/{lesson_id}/phrases",
            get(library::lesson_phrases),
        )
        .route("/api/v1/phrases/search", get(library::search))
        .route("/api/v1/phrases/{target_id}", get(library::phrase))
        .route("/api/v1/dialogues", get(library::dialogues))
        .route("/api/v1/dialogues/{dialogue_id}", get(library::dialogue))
        .route("/api/v1/sessions/readiness", get(readiness::readiness))
        .route("/api/v1/media", post(media::upload))
        .route("/api/v1/media/reconcile", post(media::reconcile))
        .route("/api/v1/media/{media_id}", get(media::play))
        .route("/api/v1/media/{media_id}/metadata", get(media::metadata));
    #[cfg(feature = "test-support")]
    let routes = routes.merge(seed::routes());
    routes
}
