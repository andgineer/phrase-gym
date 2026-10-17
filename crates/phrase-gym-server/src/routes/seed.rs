use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use phrase_gym_contracts::content::ContentPackage;
use phrase_gym_contracts::imports::ImportReport;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{ApiJson, ApiPath};
use crate::state::AppState;

// Test-only content seeding through the real validation and publication services. It is compiled
// only with the test-support feature, never into release binaries, and is absent from OpenAPI.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/imports/validate", post(validate))
        .route("/api/v1/imports", post(stage))
        .route("/api/v1/imports/{import_id}", get(staged))
        .route("/api/v1/imports/{import_id}/publish", post(publish))
}

async fn validate(
    State(state): State<AppState>,
    ApiJson(package): ApiJson<ContentPackage>,
) -> Result<Json<ImportReport>, ApiError> {
    let staging = state.staging.clone();
    Ok(Json(
        state.blocking(move || staging.validate(&package)).await??,
    ))
}

async fn stage(
    State(state): State<AppState>,
    ApiJson(package): ApiJson<ContentPackage>,
) -> Result<Json<ImportReport>, ApiError> {
    let staging = state.staging.clone();
    Ok(Json(
        state.blocking(move || staging.stage(&package)).await??,
    ))
}

async fn staged(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<ImportReport>, ApiError> {
    let staging = state.staging.clone();
    Ok(Json(state.blocking(move || staging.staged(id)).await??))
}

async fn publish(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<ImportReport>, ApiError> {
    let staging = state.staging.clone();
    Ok(Json(state.blocking(move || staging.publish(id)).await??))
}
