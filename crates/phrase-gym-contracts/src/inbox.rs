use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::Language;
use crate::english::PreparedEnglishLesson;
use crate::profiles::ProfileRevision;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct LessonPhraseInput {
    pub source_id: String,
    pub text: String,
    pub reference_audio_id: Option<Uuid>,
    pub audio_sha256: Option<String>,
    pub expected_revision: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct LessonSubmission {
    pub submission_id: Uuid,
    pub item_id: String,
    pub expected_generation: Option<i64>,
    pub language: Language,
    pub title: String,
    pub context: String,
    pub note: String,
    pub phrases: Vec<LessonPhraseInput>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Waiting,
    Preparing,
    Failed,
    PendingDelivery,
    Published,
    Cancelled,
    Superseded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LessonReceipt {
    pub generation_id: Uuid,
    pub submission_id: Uuid,
    pub generation: i64,
    pub namespace: String,
    pub profile_revision: i64,
    pub guide_version: String,
    pub received_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PublicationReceipt {
    pub generation_id: Uuid,
    pub publication_id: Uuid,
    pub lesson_id: Uuid,
    pub target_ids: Vec<Uuid>,
    pub published_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LessonStatus {
    pub receipt: LessonReceipt,
    pub status: DeliveryStatus,
    pub reason: Option<String>,
    pub publication: Option<PublicationReceipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ClaimRequest {
    pub claim_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct JobFence {
    pub generation_id: Uuid,
    pub claim_id: Uuid,
    pub token: Uuid,
    pub fence: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JobClaim {
    pub fence: JobFence,
    pub lease_until: String,
    pub deadline: String,
    pub receipt: LessonReceipt,
    pub input: LessonSubmission,
    pub profile: ProfileRevision,
    pub guide: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct LessonOutput {
    pub fence: JobFence,
    pub publication_id: Uuid,
    pub adapter: String,
    pub package: PreparedEnglishLesson,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct JobFailure {
    pub fence: JobFence,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct JobMediaRequest {
    pub fence: JobFence,
    pub media_id: Uuid,
}
