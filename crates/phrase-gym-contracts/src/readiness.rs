use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DueSummary {
    pub due_count: i64,
    pub new_count: i64,
    pub preparing_count: i64,
    pub reference_only_count: i64,
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub next_due_at: Option<String>,
}
