use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::media::{MediaAsset, MediaUpload, Reconciliation};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{ApiJson, ApiPath};
use crate::range::bytes_response;
use crate::state::AppState;

#[utoipa::path(
    post, path = "/api/v1/media", operation_id = "upload_reference_media", tag = "media",
    request_body = MediaUpload,
    responses(
        (status = 200, description = "Stored or previously stored reference media", body = MediaAsset),
        (status = 422, description = "Invalid provenance, base64 or audio container", body = ErrorBody),
    ),
)]
pub(crate) async fn upload(
    State(state): State<AppState>,
    ApiJson(upload): ApiJson<MediaUpload>,
) -> Result<Json<MediaAsset>, ApiError> {
    let ingest = state.media_ingest.clone();
    let asset = state.blocking(move || ingest.upload(&upload)).await??;
    Ok(Json(asset))
}

#[utoipa::path(
    get, path = "/api/v1/media/{media_id}/metadata", operation_id = "reference_media_metadata",
    tag = "media",
    params(("media_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Reference media provenance and availability", body = MediaAsset),
        (status = 404, description = "Reference media not found", body = ErrorBody),
        (status = 422, description = "Invalid identifier", body = ErrorBody),
    ),
)]
pub(crate) async fn metadata(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<MediaAsset>, ApiError> {
    let catalog = state.media_catalog.clone();
    let asset = state.blocking(move || catalog.asset(id)).await??;
    Ok(Json(asset))
}

#[utoipa::path(
    get, path = "/api/v1/media/{media_id}", operation_id = "play_reference_media", tag = "media",
    params(("media_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Original reference bytes"),
        (status = 206, description = "Requested byte range of the original"),
        (status = 404, description = "Reference media not found", body = ErrorBody),
        (status = 416, description = "Unsatisfiable byte range"),
        (status = 422, description = "Invalid identifier", body = ErrorBody),
        (status = 503, description = "Original is missing or damaged", body = ErrorBody),
    ),
)]
pub(crate) async fn play(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let catalog = state.media_catalog.clone();
    let delivery = state.blocking(move || catalog.delivery(id)).await??;
    Ok(bytes_response(
        delivery.bytes,
        &delivery.mime_type,
        &delivery.sha256,
        &headers,
    ))
}

#[utoipa::path(
    post, path = "/api/v1/media/reconcile", operation_id = "reconcile_reference_media",
    tag = "media",
    responses(
        (status = 200, description = "Missing and orphaned originals; nothing is deleted", body = Reconciliation),
    ),
)]
pub(crate) async fn reconcile(
    State(state): State<AppState>,
) -> Result<Json<Reconciliation>, ApiError> {
    let reconciler = state.media_reconciler.clone();
    let report = state.blocking(move || reconciler.reconcile()).await??;
    Ok(Json(report))
}
