use axum::extract::{Path, State};
use axum::{Extension, Json};
use phrase_gym_contracts::profiles::{ProfileRestore, ProfileRevision, ProfileUpdate};

use crate::actor::Actor;
use crate::error::ApiError;
use crate::extract::ApiJson;
use crate::state::AppState;

#[utoipa::path(get, path = "/api/v1/profile", tag = "profile", responses((status = 200, body = ProfileRevision)))]
pub(crate) async fn current(
    State(state): State<AppState>,
) -> Result<Json<ProfileRevision>, ApiError> {
    let profiles = state.profiles.clone();
    Ok(Json(state.blocking(move || profiles.get(None)).await??))
}

#[utoipa::path(get, path = "/api/v1/profile/revisions/{revision}", tag = "profile", params(("revision" = i64, Path)), responses((status = 200, body = ProfileRevision)))]
pub(crate) async fn revision(
    State(state): State<AppState>,
    Path(revision): Path<i64>,
) -> Result<Json<ProfileRevision>, ApiError> {
    let profiles = state.profiles.clone();
    Ok(Json(
        state
            .blocking(move || profiles.get(Some(revision)))
            .await??,
    ))
}

#[utoipa::path(put, path = "/api/v1/profile", tag = "profile", request_body = ProfileUpdate, responses((status = 200, body = ProfileRevision)))]
pub(crate) async fn update(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    ApiJson(update): ApiJson<ProfileUpdate>,
) -> Result<Json<ProfileRevision>, ApiError> {
    let profiles = state.profiles.clone();
    Ok(Json(
        state
            .blocking(move || profiles.update(&actor.actor, &update))
            .await??,
    ))
}

#[utoipa::path(post, path = "/api/v1/profile/restore", tag = "profile", request_body = ProfileRestore, responses((status = 200, body = ProfileRevision)))]
pub(crate) async fn restore(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    ApiJson(restore): ApiJson<ProfileRestore>,
) -> Result<Json<ProfileRevision>, ApiError> {
    let profiles = state.profiles.clone();
    Ok(Json(
        state
            .blocking(move || profiles.restore(&actor.actor, &restore))
            .await??,
    ))
}
