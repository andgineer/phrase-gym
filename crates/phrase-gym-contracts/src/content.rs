use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    FixedPhrase,
    WordSense,
    Frame,
}

impl TargetKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::FixedPhrase => "fixed_phrase",
            Self::WordSense => "word_sense",
            Self::Frame => "frame",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct Chunk {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct Cue {
    pub id: String,
    pub prompt: String,
    pub visible_chunks: Vec<String>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub reviewed: bool,
    #[serde(default)]
    pub audio_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct Explanation {
    pub meaning: String,
    #[serde(default)]
    pub usage: Option<String>,
    #[serde(default)]
    pub breakdown: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    pub source_id: String,
    pub kind: TargetKind,
    pub meaning_key: String,
    pub recall_expression: String,
    pub text: String,
    pub purpose: String,
    pub chunks: Vec<Chunk>,
    pub required_chunks: Vec<String>,
    #[serde(default)]
    pub cues: Vec<Cue>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = BTreeMap<String, Explanation>))]
    pub explanations: BTreeMap<Language, Explanation>,
    #[serde(default)]
    pub concepts: Vec<String>,
    #[serde(default)]
    pub acceptable_equivalents: Vec<String>,
    #[serde(default)]
    pub slots: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub reference_audio_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct LessonInput {
    pub source_id: String,
    pub title: String,
    pub target_source_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct DialogueTurn {
    pub speaker: String,
    pub text: String,
    #[serde(default)]
    pub target_source_id: Option<String>,
    #[serde(default)]
    pub audio_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct DialogueInput {
    pub source_id: String,
    pub title: String,
    pub turns: Vec<DialogueTurn>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentPackage {
    #[serde(default = "schema_version", deserialize_with = "schema_number")]
    pub schema_version: u8,
    pub namespace: String,
    pub language: Language,
    pub source_revision: String,
    pub targets: Vec<TargetInput>,
    #[serde(default)]
    pub lessons: Vec<LessonInput>,
    #[serde(default)]
    pub dialogues: Vec<DialogueInput>,
}

fn schema_version() -> u8 {
    1
}

fn schema_number<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    match u8::deserialize(deserializer)? {
        1 => Ok(1),
        other => Err(serde::de::Error::custom(format!(
            "unsupported package schema_version {other}"
        ))),
    }
}
