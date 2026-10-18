use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::base::Language;
use crate::library::{Dialogue, Phrase};
use crate::practice::{AttemptEvidence, EventReceipt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Paused,
    Superseded,
    Completed,
}

impl SessionStatus {
    pub fn code(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Superseded => "superseded",
            Self::Completed => "completed",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        [
            Self::Active,
            Self::Paused,
            Self::Superseded,
            Self::Completed,
        ]
        .into_iter()
        .find(|status| status.code() == code)
    }

    pub fn is_open(self) -> bool {
        matches!(self, Self::Active | Self::Paused)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum OpenStatus {
    Active,
    Paused,
}

impl From<OpenStatus> for SessionStatus {
    fn from(status: OpenStatus) -> Self {
        match status {
            OpenStatus::Active => Self::Active,
            OpenStatus::Paused => Self::Paused,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SessionStart {
    pub id: Uuid,
    pub language: Language,
    #[serde(default)]
    pub lesson_id: Option<Uuid>,
    #[serde(default)]
    pub dialogue_id: Option<Uuid>,
    #[serde(default)]
    pub takeover_session_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SessionTransition {
    #[cfg_attr(feature = "openapi", schema(minimum = 1))]
    pub expected_version: i64,
    pub status: OpenStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum Eligibility {
    New,
    Due,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlannedTarget {
    pub phrase: Phrase,
    pub cue_id: String,
    pub hidden_chunks: Vec<String>,
    pub review_group_id: Uuid,
    #[cfg_attr(feature = "openapi", schema(minimum = 0))]
    pub schedule_version: i64,
    pub eligibility: Eligibility,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManifestSchema;

impl Serialize for ManifestSchema {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(1)
    }
}

impl<'de> Deserialize<'de> for ManifestSchema {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match u8::deserialize(deserializer)? {
            1 => Ok(Self),
            other => Err(serde::de::Error::custom(format!(
                "unsupported manifest schema_version {other}"
            ))),
        }
    }
}

#[cfg(feature = "openapi")]
impl utoipa::PartialSchema for ManifestSchema {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Integer)
            .enum_values(Some([1]))
            .into()
    }
}

#[cfg(feature = "openapi")]
impl utoipa::ToSchema for ManifestSchema {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PlannerVersion {
    #[default]
    #[serde(rename = "due-new-v3-online")]
    DueNewV3Online,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SessionManifest {
    #[cfg_attr(feature = "openapi", schema(inline))]
    pub schema_version: ManifestSchema,
    pub planner_version: PlannerVersion,
    pub policy_version: String,
    pub language: Language,
    pub lesson_id: Option<Uuid>,
    pub targets: Vec<PlannedTarget>,
    pub required_media_ids: Vec<Uuid>,
    pub dialogue: Option<Dialogue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SessionStep {
    pub target_index: usize,
    pub repair: bool,
    pub dialogue_turn: Option<usize>,
    pub introduced: bool,
}

impl SessionStep {
    pub fn new(target_index: usize) -> Self {
        Self {
            target_index,
            repair: false,
            dialogue_turn: None,
            introduced: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PracticeSession {
    pub id: Uuid,
    pub status: SessionStatus,
    #[cfg_attr(feature = "openapi", schema(minimum = 1))]
    pub version: i64,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub created_at: String,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub updated_at: String,
    pub manifest: SessionManifest,
    pub cursor: i64,
    pub last_sequence: i64,
    pub queue: Vec<SessionStep>,
    pub attempts: Vec<AttemptEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PracticeBatchReceipt {
    pub receipts: Vec<EventReceipt>,
    pub session: PracticeSession,
}

#[cfg(test)]
mod tests {
    use super::PlannerVersion;

    #[test]
    fn only_the_current_planner_version_decodes() {
        let current = PlannerVersion::DueNewV3Online;
        let encoded = serde_json::to_value(current).unwrap();
        assert_eq!(encoded, "due-new-v3-online");
        assert_eq!(
            serde_json::from_value::<PlannerVersion>(encoded).unwrap(),
            current
        );
        for retired in ["due-new-v1", "due-new-v2"] {
            assert!(serde_json::from_value::<PlannerVersion>(retired.into()).is_err());
        }
    }
}
