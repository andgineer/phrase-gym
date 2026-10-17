use phrase_gym_contracts::base::Language;
use phrase_gym_core::library::Paging;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::error::ApiError;

const MAX_SEARCH: usize = 200;
const MAX_LIMIT: i64 = 100;

fn default_limit() -> i64 {
    50
}

fn invalid(detail: &str) -> ApiError {
    ApiError::new(axum::http::StatusCode::UNPROCESSABLE_ENTITY, detail)
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct PageQuery {
    pub language: Language,
    #[serde(default)]
    #[param(minimum = 0)]
    pub offset: i64,
    #[serde(default = "default_limit")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: i64,
}

impl PageQuery {
    pub fn paging(&self) -> Result<Paging, ApiError> {
        if self.offset < 0 || !(1..=MAX_LIMIT).contains(&self.limit) {
            return Err(invalid("offset must be at least 0 and limit 1-100"));
        }
        Ok(Paging {
            offset: self.offset,
            limit: self.limit,
        })
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct SearchQuery {
    pub language: Language,
    #[serde(default)]
    #[param(max_length = 200)]
    pub q: String,
    #[serde(default)]
    #[param(minimum = 0)]
    pub offset: i64,
    #[serde(default = "default_limit")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: i64,
}

impl SearchQuery {
    pub fn paging(&self) -> Result<Paging, ApiError> {
        if self.q.chars().count() > MAX_SEARCH {
            return Err(invalid("q must be at most 200 characters"));
        }
        PageQuery {
            language: self.language,
            offset: self.offset,
            limit: self.limit,
        }
        .paging()
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct RevisionQuery {
    pub language: Language,
    #[param(minimum = 1, nullable)]
    pub revision: Option<i64>,
}

impl RevisionQuery {
    pub fn revision(&self) -> Result<Option<i64>, ApiError> {
        match self.revision {
            Some(revision) if revision < 1 => Err(invalid("revision must be at least 1")),
            revision => Ok(revision),
        }
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct ReadinessQuery {
    pub language: Language,
    #[param(nullable)]
    pub lesson_id: Option<Uuid>,
}
