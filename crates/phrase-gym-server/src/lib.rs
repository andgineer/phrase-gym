mod error;
mod extract;
pub mod openapi;
mod routes;
mod state;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;

pub use state::AppState;

const MAX_REQUEST_BYTES: usize = 24 * 1024 * 1024;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/exposures", post(routes::exposures::expose))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .with_state(state)
}
