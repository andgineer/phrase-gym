use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::Language;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct LanguagePreferences {
    pub proficiency: Option<String>,
    pub card_language: Language,
    pub help_language: Option<Language>,
    pub preparation_guidance: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PersonalProfile {
    pub native_language: Language,
    pub learning_timezone: String,
    #[cfg_attr(feature = "openapi", schema(value_type = BTreeMap<String, LanguagePreferences>))]
    pub languages: BTreeMap<Language, LanguagePreferences>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ProfileUpdate {
    pub mutation_id: Uuid,
    pub expected_revision: i64,
    pub profile: PersonalProfile,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ProfileRestore {
    pub mutation_id: Uuid,
    pub expected_revision: i64,
    pub restore_revision: i64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProfileRevision {
    pub revision: i64,
    pub profile: PersonalProfile,
    pub mutation_id: Option<Uuid>,
    pub actor: String,
    pub reason: String,
    pub created_at: String,
    pub restored_from: Option<i64>,
}
