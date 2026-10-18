use axum::Json;
use axum::extract::State;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::practice::{EventBatch, EventReceipt};
use phrase_gym_contracts::sessions::PracticeBatchReceipt;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{ApiJson, ApiPath};
use crate::state::AppState;

#[utoipa::path(
    post, path = "/api/v1/sessions/{session_id}/events", operation_id = "submit_practice_events",
    tag = "practice",
    params(("session_id" = Uuid, Path)),
    request_body = EventBatch,
    responses(
        (status = 200, description = "Receipts for every event, including retried ones, and the session", body = PracticeBatchReceipt),
        (status = 404, description = "Session not found", body = ErrorBody),
        (status = 409, description = "Out-of-order event, stale step or event ID reused with another payload; nothing in the batch is kept", body = ErrorBody),
        (status = 422, description = "Invalid event", body = ErrorBody),
    ),
)]
pub(crate) async fn submit(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(batch): ApiJson<EventBatch>,
) -> Result<Json<PracticeBatchReceipt>, ApiError> {
    let practice = state.practice.clone();
    Ok(Json(
        state
            .blocking(move || practice.submit(id, &batch))
            .await??,
    ))
}

#[utoipa::path(
    get, path = "/api/v1/sessions/{session_id}/events", operation_id = "practice_event_receipts",
    tag = "practice",
    params(("session_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Receipts in event order", body = Vec<EventReceipt>),
        (status = 404, description = "Session not found", body = ErrorBody),
        (status = 422, description = "Invalid identifier", body = ErrorBody),
    ),
)]
pub(crate) async fn receipts(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<Vec<EventReceipt>>, ApiError> {
    let practice = state.practice.clone();
    Ok(Json(state.blocking(move || practice.receipts(id)).await??))
}
