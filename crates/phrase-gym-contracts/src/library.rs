use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::Language;
use crate::content::{DialogueInput, TargetInput};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Lesson {
    pub id: Uuid,
    pub language: Language,
    pub title: String,
    pub phrase_count: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum AudioStatus {
    Ready,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Phrase {
    pub id: Uuid,
    pub revision: i64,
    pub language: Language,
    pub content: TargetInput,
    pub lessons: Vec<Lesson>,
    pub audio_status: AudioStatus,
    pub retrieval_ready: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub english: Option<crate::english::PreparedEnglishPhrase>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Dialogue {
    pub id: Uuid,
    pub revision: i64,
    pub language: Language,
    pub content: DialogueInput,
    pub target_ids: BTreeMap<String, Uuid>,
    pub target_revisions: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PhrasePage {
    pub items: Vec<Phrase>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LessonPage {
    pub items: Vec<Lesson>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DialoguePage {
    pub items: Vec<Dialogue>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}
