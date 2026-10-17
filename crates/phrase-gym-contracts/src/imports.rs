use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeAction {
    Create,
    Update,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportChange {
    pub source_id: String,
    pub action: ChangeAction,
    pub target_id: Option<Uuid>,
    pub base_revision: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportStatus {
    Validated,
    Staged,
    Published,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportReport {
    pub id: Option<Uuid>,
    pub status: ImportStatus,
    pub changes: Vec<ImportChange>,
    pub errors: Vec<String>,
    pub base_content_version: String,
}
