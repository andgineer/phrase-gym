use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::recordings::{RecordingInput, RecordingReceipt};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{ApiJson, ApiPath};
use crate::range::bytes_response;
use crate::state::AppState;

#[utoipa::path(
    post, path = "/api/v1/recordings", operation_id = "store_learner_take", tag = "recordings",
    request_body = RecordingInput,
    responses(
        (status = 200, description = "Stored take, or the take already stored with this ID", body = RecordingReceipt),
        (status = 409, description = "Take ID reused with other audio or metadata, or its attempt or capture event is not persisted", body = ErrorBody),
        (status = 422, description = "Invalid transfer, MIME type, size or context", body = ErrorBody),
    ),
)]
pub(crate) async fn store(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<RecordingInput>,
) -> Result<Json<RecordingReceipt>, ApiError> {
    let recordings = state.recordings.clone();
    Ok(Json(
        state.blocking(move || recordings.save(&input)).await??,
    ))
}

#[utoipa::path(
    get, path = "/api/v1/phrases/{target_id}/recordings", operation_id = "list_learner_takes",
    tag = "recordings",
    params(("target_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Every take of the phrase in capture order", body = Vec<RecordingReceipt>),
        (status = 422, description = "Invalid identifier", body = ErrorBody),
    ),
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    ApiPath(target): ApiPath<Uuid>,
) -> Result<Json<Vec<RecordingReceipt>>, ApiError> {
    let recordings = state.recordings.clone();
    Ok(Json(
        state.blocking(move || recordings.list(target)).await??,
    ))
}

#[utoipa::path(
    get, path = "/api/v1/recordings/{recording_id}", operation_id = "play_learner_take",
    tag = "recordings",
    params(("recording_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Original take bytes"),
        (status = 206, description = "Requested byte range of the original take"),
        (status = 404, description = "Take not found", body = ErrorBody),
        (status = 416, description = "Unsatisfiable byte range"),
        (status = 422, description = "Invalid identifier", body = ErrorBody),
    ),
)]
pub(crate) async fn play(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let recordings = state.recordings.clone();
    let delivery = state.blocking(move || recordings.delivery(id)).await??;
    Ok(bytes_response(
        delivery.bytes,
        &delivery.mime_type,
        &delivery.sha256,
        &headers,
    ))
}
