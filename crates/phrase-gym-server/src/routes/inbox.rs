use axum::extract::State;
use axum::routing::post;
use axum::{Extension, Json, Router};
use phrase_gym_contracts::inbox::{LessonReceipt, LessonStatus, LessonSubmission};
use uuid::Uuid;

use crate::authority::Credential;
use crate::error::ApiError;
use crate::extract::{ApiJson, ApiPath};
use crate::state::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/inbox/lessons", post(submit))
        .route("/api/v1/inbox/lessons/{id}/cancel", post(cancel))
        .route("/api/v1/inbox/lessons/{id}/retry", post(retry))
}

#[utoipa::path(post, path = "/api/v1/inbox/lessons", tag = "inbox", request_body = LessonSubmission, responses((status = 200, body = LessonReceipt)))]
pub(crate) async fn submit(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiJson(input): ApiJson<LessonSubmission>,
) -> Result<Json<LessonReceipt>, ApiError> {
    let inbox = state.inbox.clone();
    Ok(Json(
        state
            .blocking(move || inbox.submit(&actor.actor, &input))
            .await??,
    ))
}

#[utoipa::path(get, path = "/api/v1/inbox/lessons/{id}", tag = "inbox", params(("id" = Uuid, Path)), responses((status = 200, body = LessonStatus)))]
pub(crate) async fn status(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<LessonStatus>, ApiError> {
    let inbox = state.inbox.clone();
    Ok(Json(
        state
            .blocking(move || inbox.status(&actor.actor, id))
            .await??,
    ))
}

#[utoipa::path(post, path = "/api/v1/inbox/lessons/{id}/cancel", tag = "inbox", params(("id" = Uuid, Path)), responses((status = 200, body = LessonStatus)))]
pub(crate) async fn cancel(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<LessonStatus>, ApiError> {
    let inbox = state.inbox.clone();
    Ok(Json(
        state
            .blocking(move || inbox.cancel(&actor.actor, id))
            .await??,
    ))
}

#[utoipa::path(post, path = "/api/v1/inbox/lessons/{id}/retry", tag = "inbox", params(("id" = Uuid, Path)), responses((status = 200, body = LessonStatus)))]
pub(crate) async fn retry(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<LessonStatus>, ApiError> {
    let inbox = state.inbox.clone();
    Ok(Json(
        state
            .blocking(move || inbox.retry(&actor.actor, id))
            .await??,
    ))
}
