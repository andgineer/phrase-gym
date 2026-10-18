use axum::Json;
use axum::extract::State;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::sessions::{PracticeSession, SessionStart, SessionTransition};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::{ApiJson, ApiPath};
use crate::state::AppState;

#[utoipa::path(
    post, path = "/api/v1/sessions", operation_id = "start_session", tag = "sessions",
    request_body = SessionStart,
    responses(
        (status = 200, description = "Started session, or the session already started with this ID", body = PracticeSession),
        (status = 404, description = "Lesson or dialogue not found in this learning language", body = ErrorBody),
        (status = 409, description = "Another session is open, the ID has other parameters or nothing is ready", body = ErrorBody),
        (status = 422, description = "Invalid start parameters", body = ErrorBody),
    ),
)]
pub(crate) async fn start(
    State(state): State<AppState>,
    ApiJson(start): ApiJson<SessionStart>,
) -> Result<Json<PracticeSession>, ApiError> {
    let sessions = state.sessions.clone();
    Ok(Json(state.blocking(move || sessions.start(&start)).await??))
}

#[utoipa::path(
    get, path = "/api/v1/sessions/current", operation_id = "current_session", tag = "sessions",
    responses(
        (status = 200, description = "The open session, or null when none is open", body = Option<PracticeSession>),
    ),
)]
pub(crate) async fn current(
    State(state): State<AppState>,
) -> Result<Json<Option<PracticeSession>>, ApiError> {
    let sessions = state.sessions.clone();
    Ok(Json(state.blocking(move || sessions.current()).await??))
}

#[utoipa::path(
    get, path = "/api/v1/sessions/{session_id}", operation_id = "read_session", tag = "sessions",
    params(("session_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Session with its immutable manifest, position and attempts", body = PracticeSession),
        (status = 404, description = "Session not found", body = ErrorBody),
        (status = 422, description = "Invalid identifier", body = ErrorBody),
    ),
)]
pub(crate) async fn read(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<PracticeSession>, ApiError> {
    let sessions = state.sessions.clone();
    Ok(Json(state.blocking(move || sessions.get(id)).await??))
}

#[utoipa::path(
    patch, path = "/api/v1/sessions/{session_id}", operation_id = "transition_session",
    tag = "sessions",
    params(("session_id" = Uuid, Path)),
    request_body = SessionTransition,
    responses(
        (status = 200, description = "Paused or resumed session", body = PracticeSession),
        (status = 404, description = "Session not found", body = ErrorBody),
        (status = 409, description = "Closed session or stale expected version", body = ErrorBody),
        (status = 422, description = "Invalid transition", body = ErrorBody),
    ),
)]
pub(crate) async fn transition(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(transition): ApiJson<SessionTransition>,
) -> Result<Json<PracticeSession>, ApiError> {
    let sessions = state.sessions.clone();
    Ok(Json(
        state
            .blocking(move || sessions.transition(id, &transition))
            .await??,
    ))
}
