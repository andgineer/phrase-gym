use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::sync::watch;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extract::ApiPath;
use crate::state::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/_test/reset", post(reset))
        .route("/_test/pending-delivery/{enabled}", post(pending))
        .route("/_test/hold-recording-ack", post(hold_recording))
        .route("/_test/release-recording-ack", post(release_recording))
        .route("/_test/lose-rating", post(lose_rating))
        .route("/_test/acknowledge-rating", post(acknowledge_rating))
        .route("/_test/hold-session-read/{id}", post(hold_session))
        .route("/_test/release-session-read", post(release_session))
        .route("/_test/session-read-state", get(session_state))
}

async fn reset(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let controls = state.browser.clone();
    let profiles = state.profiles.clone();
    state
        .blocking(move || {
            controls.reset().map_err(|_| ApiError::internal())?;
            profiles
                .initialize_current_learner()
                .map_err(ApiError::from)
        })
        .await??;
    Ok(Json(json!({"reset": true})))
}

async fn pending(State(state): State<AppState>, ApiPath(enabled): ApiPath<bool>) -> Json<Value> {
    state.browser.lock().pending = enabled;
    Json(json!({"held": enabled}))
}

async fn hold_recording(State(state): State<AppState>) -> Json<Value> {
    state.browser.lock().recording = Some(watch::channel(false).0);
    Json(json!({"held": true}))
}

async fn release_recording(State(state): State<AppState>) -> Json<Value> {
    if let Some(gate) = state.browser.lock().recording.take() {
        let _ = gate.send(true);
    }
    Json(json!({"held": false}))
}

async fn lose_rating(State(state): State<AppState>) -> Json<Value> {
    state.browser.lock().lose_rating = true;
    Json(json!({"armed": true}))
}

async fn acknowledge_rating(State(state): State<AppState>) -> Json<Value> {
    state.browser.lock().lose_rating = false;
    Json(json!({"armed": false}))
}

async fn hold_session(State(state): State<AppState>, ApiPath(id): ApiPath<Uuid>) -> Json<Value> {
    let mut faults = state.browser.lock();
    faults.session = Some((Some(id), watch::channel(false).0));
    faults.session_held = false;
    Json(json!({"armed": true}))
}

async fn release_session(State(state): State<AppState>) -> Json<Value> {
    if let Some((_, gate)) = state.browser.lock().session.take() {
        let _ = gate.send(true);
    }
    Json(json!({"released": true}))
}

async fn session_state(State(state): State<AppState>) -> Json<Value> {
    Json(json!({"held": state.browser.lock().session_held}))
}
