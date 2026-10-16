use axum::Json;
use axum::extract::State;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::exposures::{ExposureInput, ExposureReceipt};

use crate::error::ApiError;
use crate::extract::ApiJson;
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/api/v1/exposures",
    operation_id = "record_exposure",
    tag = "exposures",
    request_body = ExposureInput,
    responses(
        (status = 200, description = "Recorded or previously recorded exposure", body = ExposureReceipt),
        (status = 409, description = "Exposure ID already used with a different payload", body = ErrorBody),
        (status = 422, description = "Invalid payload or unknown phrase revision", body = ErrorBody),
    ),
)]
pub(crate) async fn expose(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<ExposureInput>,
) -> Result<Json<ExposureReceipt>, ApiError> {
    let service = state.exposures.clone();
    let receipt = state.storage(move || service.expose(&input)).await??;
    Ok(Json(receipt))
}
