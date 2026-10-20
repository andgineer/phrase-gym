use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum RecallRating {
    NotYet,
    WithEffort,
    Recalled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    #[default]
    SelfReport,
    Recorded,
    Unrecorded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum PracticeKind {
    Next,
    Present,
    Respond,
    CaptureComplete,
    Answer,
    Explain,
    Reference,
    Rate,
    Failure,
    Report,
    Detour,
    DialogueExposure,
    DialogueBegin,
}

impl PracticeKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Present => "present",
            Self::Respond => "respond",
            Self::CaptureComplete => "capture_complete",
            Self::Answer => "answer",
            Self::Explain => "explain",
            Self::Reference => "reference",
            Self::Rate => "rate",
            Self::Failure => "failure",
            Self::Report => "report",
            Self::Detour => "detour",
            Self::DialogueExposure => "dialogue_exposure",
            Self::DialogueBegin => "dialogue_begin",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PracticeInput {
    pub id: Uuid,
    #[cfg_attr(feature = "openapi", schema(minimum = 1))]
    pub sequence: i64,
    #[cfg_attr(feature = "openapi", schema(minimum = 0))]
    pub step: i64,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub occurred_at: String,
    pub kind: PracticeKind,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = "self_report"))]
    pub modality: Modality,
    #[serde(default)]
    pub rating: Option<RecallRating>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(max_length = 1000))]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EventBatch {
    #[cfg_attr(feature = "openapi", schema(min_items = 1, max_items = 100))]
    pub events: Vec<PracticeInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OriginalResponse {
    pub modality: Modality,
    pub supported: bool,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub occurred_at: String,
}

// Independent evidence flags the client renders; they are not states of one machine.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttemptEvidence {
    pub id: Uuid,
    pub step: i64,
    pub target_id: Uuid,
    pub revision: i64,
    pub cue_id: String,
    pub review_group_id: Uuid,
    pub repair: bool,
    pub supported: bool,
    pub original: Option<OriginalResponse>,
    pub revealed: bool,
    pub expanded: bool,
    pub technical_failure: bool,
    pub capture_pending: bool,
    pub rating: Option<RecallRating>,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub presented_at: String,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub finalized_at: Option<String>,
    pub eligibility_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EventReceipt {
    pub id: Uuid,
    pub sequence: i64,
    pub attempt_id: Uuid,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub received_at: String,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub occurred_at: String,
    pub clock_adjusted: bool,
    pub capture_supported: Option<bool>,
    pub schedule_changed: bool,
    pub schedule_version: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub due_at: Option<String>,
    pub eligibility_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_default_to_self_report_and_reject_unknown_fields() {
        let event = serde_json::json!({
            "id": "000e7e47-0000-4000-8000-000000000001",
            "sequence": 1,
            "step": 0,
            "kind": "present",
            "occurred_at": "2026-10-12T12:00:00+00:00",
        });
        let parsed: PracticeInput = serde_json::from_value(event.clone()).unwrap();
        assert_eq!(parsed.modality, Modality::SelfReport);
        assert_eq!(parsed.rating, None);
        let mut retired = event;
        retired["defer_repairs"] = true.into();
        assert!(serde_json::from_value::<PracticeInput>(retired).is_err());
    }
}
