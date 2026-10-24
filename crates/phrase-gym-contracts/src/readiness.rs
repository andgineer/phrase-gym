use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DueSummary {
    pub practice_ready_count: i64,
    pub due_count: i64,
    pub new_count: i64,
    pub preparing_count: i64,
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub next_due_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CoachSummary {
    pub lessons: i64,
    pub targets: i64,
    pub ready: i64,
    pub study_only: i64,
    pub awaiting_audio: i64,
}
