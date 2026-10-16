use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_core::error::CoreError;

#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    detail: String,
}

impl ApiError {
    pub(crate) fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
        }
    }

    pub(crate) fn unavailable() -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, "Server is shutting down")
    }

    pub(crate) fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::NotFound(detail) => Self::new(StatusCode::NOT_FOUND, detail),
            CoreError::Conflict(detail) => Self::new(StatusCode::CONFLICT, detail),
            CoreError::Invalid(detail) => Self::new(StatusCode::UNPROCESSABLE_ENTITY, detail),
            CoreError::Storage(error) => {
                tracing::error!(%error, "storage failure");
                Self::internal()
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                detail: self.detail,
            }),
        )
            .into_response()
    }
}
