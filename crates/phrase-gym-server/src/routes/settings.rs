use axum::Json;
use axum::extract::State;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::settings::{Settings, SettingsUpdate};

use crate::error::ApiError;
use crate::extract::ApiJson;
use crate::state::AppState;

#[utoipa::path(
    get, path = "/api/v1/settings", operation_id = "read_device_settings", tag = "settings",
    responses(
        (status = 200, description = "Device and interface settings, with defaults before the first save", body = Settings),
    ),
)]
pub(crate) async fn read(State(state): State<AppState>) -> Result<Json<Settings>, ApiError> {
    let settings = state.settings.clone();
    Ok(Json(state.blocking(move || settings.current()).await??))
}

#[utoipa::path(
    put, path = "/api/v1/settings", operation_id = "replace_device_settings", tag = "settings",
    request_body = SettingsUpdate,
    responses(
        (status = 200, description = "The complete replacement settings", body = Settings),
        (status = 422, description = "Invalid or unknown settings", body = ErrorBody),
    ),
)]
pub(crate) async fn replace(
    State(state): State<AppState>,
    ApiJson(update): ApiJson<SettingsUpdate>,
) -> Result<Json<Settings>, ApiError> {
    let settings = state.settings.clone();
    Ok(Json(
        state.blocking(move || settings.replace(&update)).await??,
    ))
}
