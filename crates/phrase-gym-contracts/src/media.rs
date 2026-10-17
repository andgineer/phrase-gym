use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum MediaRole {
    Reference,
    Cue,
    Partner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum MediaOrigin {
    Actor,
    Course,
    Generated,
    Test,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum SettingValue {
    Text(String),
    Integer(i64),
    Number(f64),
    Flag(bool),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct MediaMetadata {
    pub language: Language,
    pub text: String,
    pub role: MediaRole,
    pub origin: MediaOrigin,
    pub source: String,
    #[serde(default)]
    pub performer: Option<String>,
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub voice: Option<String>,
    pub locale: String,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub speech_text: Option<String>,
    pub preparation_version: String,
    #[serde(default)]
    pub settings: BTreeMap<String, SettingValue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ReferenceMime {
    #[serde(rename = "audio/wav")]
    Wav,
    #[serde(rename = "audio/mpeg")]
    Mpeg,
    #[serde(rename = "audio/ogg")]
    Ogg,
    #[serde(rename = "audio/mp4")]
    Mp4,
}

impl ReferenceMime {
    pub fn code(self) -> &'static str {
        match self {
            Self::Wav => "audio/wav",
            Self::Mpeg => "audio/mpeg",
            Self::Ogg => "audio/ogg",
            Self::Mp4 => "audio/mp4",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct MediaUpload {
    pub metadata: MediaMetadata,
    pub mime_type: ReferenceMime,
    pub audio_base64: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MediaAsset {
    pub id: Uuid,
    pub sha256: String,
    pub byte_length: i64,
    pub mime_type: String,
    pub metadata: MediaMetadata,
    pub available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Reconciliation {
    pub missing_ids: Vec<Uuid>,
    pub orphan_hashes: Vec<String>,
    pub removed_staging_files: i64,
}
