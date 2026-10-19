use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::Language;
use crate::content::LessonInput;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum EnglishPackageVersion {
    #[serde(rename = "english_phrases_v1")]
    EnglishPhrasesV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PartnerPrompt {
    pub text: String,
    pub audio_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PhraseSituation {
    pub id: String,
    pub text: String,
    pub language: Language,
    pub audio_id: Option<Uuid>,
    pub partner: Option<PartnerPrompt>,
    pub exposed_target_source_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PhraseHelp {
    pub language: Language,
    pub meaning: String,
    pub usage: Option<String>,
    pub translation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PhraseFamiliarity {
    New,
    Studied { provenance: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PreparedEnglishPhrase {
    pub source_id: String,
    pub meaning_key: String,
    pub intention_key: String,
    pub text: String,
    pub accepted_variants: Vec<String>,
    pub situations: Vec<PhraseSituation>,
    pub hint: String,
    pub help: Option<PhraseHelp>,
    pub familiarity: PhraseFamiliarity,
    pub reference_audio_id: Uuid,
    pub expected_revision: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PreparedEnglishLesson {
    pub schema_version: EnglishPackageVersion,
    pub namespace: String,
    pub source_revision: String,
    pub language: Language,
    pub profile_revision: i64,
    pub guide_version: String,
    pub assessment_version: String,
    pub lesson: LessonInput,
    pub phrases: Vec<PreparedEnglishPhrase>,
}
