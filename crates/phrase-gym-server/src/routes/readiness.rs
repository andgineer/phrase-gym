use axum::Json;
use axum::extract::State;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::readiness::DueSummary;

use super::params::ReadinessQuery;
use crate::error::ApiError;
use crate::extract::ApiQuery;
use crate::state::AppState;

#[utoipa::path(
    get, path = "/api/v1/sessions/readiness", operation_id = "practice_readiness",
    tag = "sessions",
    params(ReadinessQuery),
    responses(
        (status = 200, description = "Due, new and preparing counts without starting a session", body = DueSummary),
        (status = 404, description = "Lesson not found in this learning language", body = ErrorBody),
        (status = 422, description = "Invalid query", body = ErrorBody),
    ),
)]
pub(crate) async fn readiness(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<ReadinessQuery>,
) -> Result<Json<DueSummary>, ApiError> {
    let readiness = state.readiness.clone();
    let summary = state
        .blocking(move || readiness.summary(query.language, query.lesson_id))
        .await??;
    Ok(Json(summary))
}
