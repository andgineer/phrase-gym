use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::post;
use axum::{Extension, Json, Router};
use phrase_gym_contracts::inbox::JobMediaRequest;
use phrase_gym_contracts::inbox::{
    ClaimRequest, JobClaim, JobFailure, JobFence, LessonOutput, PublicationReceipt,
};

use crate::authority::Credential;
use crate::error::ApiError;
use crate::extract::ApiJson;
use crate::state::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/preparation/claims", post(claim))
        .route("/api/v1/preparation/audio", post(audio))
        .route("/api/v1/preparation/renew", post(renew))
        .route("/api/v1/preparation/fail", post(fail))
        .route("/api/v1/preparation/stage", post(stage))
        .route("/api/v1/preparation/publish", post(publish))
        .route("/api/v1/preparation/receipt", post(receipt))
}

#[utoipa::path(post, path = "/api/v1/preparation/claims", tag = "preparation", request_body = ClaimRequest, responses((status = 200, body = Option<JobClaim>)))]
pub(crate) async fn claim(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiJson(input): ApiJson<ClaimRequest>,
) -> Result<Json<Option<JobClaim>>, ApiError> {
    let jobs = state.jobs.clone();
    Ok(Json(
        state
            .blocking(move || jobs.claim(&actor.actor, input.claim_id))
            .await??,
    ))
}

#[utoipa::path(post, path = "/api/v1/preparation/renew", tag = "preparation", request_body = JobFence, responses((status = 200, body = String)))]
pub(crate) async fn renew(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiJson(input): ApiJson<JobFence>,
) -> Result<Json<String>, ApiError> {
    let jobs = state.jobs.clone();
    Ok(Json(
        state
            .blocking(move || jobs.renew(&actor.actor, &input))
            .await??,
    ))
}

#[utoipa::path(post, path = "/api/v1/preparation/fail", tag = "preparation", request_body = JobFailure, responses((status = 200)))]
pub(crate) async fn fail(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiJson(input): ApiJson<JobFailure>,
) -> Result<Json<()>, ApiError> {
    let jobs = state.jobs.clone();
    state
        .blocking(move || jobs.fail(&actor.actor, &input))
        .await??;
    Ok(Json(()))
}

#[utoipa::path(post, path = "/api/v1/preparation/stage", tag = "preparation", request_body = LessonOutput, responses((status = 200)))]
pub(crate) async fn stage(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiJson(input): ApiJson<LessonOutput>,
) -> Result<Json<()>, ApiError> {
    let delivery = state.delivery.clone();
    state
        .blocking(move || delivery.stage(&actor.actor, &input))
        .await??;
    Ok(Json(()))
}

#[utoipa::path(post, path = "/api/v1/preparation/publish", tag = "preparation", request_body = JobFence, responses((status = 200, body = PublicationReceipt)))]
pub(crate) async fn publish(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiJson(input): ApiJson<JobFence>,
) -> Result<Json<PublicationReceipt>, ApiError> {
    let delivery = state.delivery.clone();
    Ok(Json(
        state
            .blocking(move || delivery.publish(&actor.actor, &input))
            .await??,
    ))
}

#[utoipa::path(post, path = "/api/v1/preparation/receipt", tag = "preparation", request_body = JobFence, responses((status = 200, body = Option<PublicationReceipt>)))]
pub(crate) async fn receipt(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    ApiJson(input): ApiJson<JobFence>,
) -> Result<Json<Option<PublicationReceipt>>, ApiError> {
    let delivery = state.delivery.clone();
    Ok(Json(
        state
            .blocking(move || delivery.receipt(&actor.actor, &input))
            .await??,
    ))
}

#[utoipa::path(post, path = "/api/v1/preparation/audio", tag = "preparation", request_body = JobMediaRequest, responses((status = 200, description = "Original audio bytes for this active job")))]
pub(crate) async fn audio(
    State(state): State<AppState>,
    Extension(actor): Extension<Credential>,
    headers: HeaderMap,
    ApiJson(input): ApiJson<JobMediaRequest>,
) -> Result<Response, ApiError> {
    let jobs = state.jobs.clone();
    let catalog = state.media_catalog.clone();
    let delivery = state
        .blocking(move || {
            jobs.authorize_media(&actor.actor, &input.fence, input.media_id)?;
            catalog.delivery(input.media_id)
        })
        .await??;
    Ok(crate::range::bytes_response(
        delivery.bytes,
        &delivery.mime_type,
        &delivery.sha256,
        &headers,
    ))
}
