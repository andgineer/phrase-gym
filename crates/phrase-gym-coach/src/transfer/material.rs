use std::collections::BTreeMap;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::inbox::{LessonPhraseInput, LessonSubmission};
use phrase_gym_contracts::media::{
    MediaAsset, MediaMetadata, MediaOrigin, MediaRole, ReferenceMime,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::supplied::SuppliedLesson;

#[derive(Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "content",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Material {
    Supplied(Box<SuppliedLesson>),
    Raw(RawLesson),
    Generated {
        phrase: String,
        metadata: MediaMetadata,
    },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLesson {
    pub title: String,
    pub phrase: String,
    pub situation: String,
    pub performer: String,
    pub locale: String,
}

impl RawLesson {
    pub fn validate(&self) -> Result<(), String> {
        for (value, limit) in [
            (&self.title, 200),
            (&self.phrase, 200),
            (&self.situation, 800),
            (&self.performer, 200),
            (&self.locale, 30),
        ] {
            if value.trim().is_empty()
                || value.trim() != value
                || value.len() > limit
                || value.chars().any(char::is_control)
            {
                return Err("Complete the title, exact English phrase, situation, performer and audio locale within the displayed limits".into());
            }
        }
        if self.phrase.contains("http://") || self.phrase.contains("https://") {
            return Err("Paste an extracted English phrase, not a URL".into());
        }
        if !self.locale.starts_with("en-") {
            return Err("Choose the supplied clip's English locale, such as en-US".into());
        }
        Ok(())
    }
}

impl Material {
    pub fn metadata(&self) -> MediaMetadata {
        match self {
            Self::Supplied(supplied) => supplied.metadata.clone(),
            Self::Generated { metadata, .. } => metadata.clone(),
            Self::Raw(raw) => MediaMetadata {
                language: Language::En,
                text: raw.phrase.clone(),
                role: MediaRole::Reference,
                origin: MediaOrigin::Actor,
                source: "coach-deliberate-input".into(),
                performer: Some(raw.performer.clone()),
                engine: None,
                voice: None,
                locale: raw.locale.clone(),
                accent: None,
                speech_text: None,
                preparation_version: "coach-raw-english-v1".into(),
                settings: BTreeMap::default(),
            },
        }
    }

    pub fn mime(&self) -> ReferenceMime {
        match self {
            Self::Supplied(s) => s.mime_type,
            Self::Raw(_) | Self::Generated { .. } => ReferenceMime::Wav,
        }
    }

    pub fn submission(&self, id: Uuid, media: &MediaAsset) -> Result<LessonSubmission, String> {
        if let Self::Supplied(supplied) = self {
            return supplied.submission(id, media);
        }
        let (title, phrase, context) = match self {
            Self::Raw(raw) => {
                raw.validate()?;
                (raw.title.clone(), raw.phrase.clone(), raw.situation.clone())
            }
            Self::Generated { phrase, .. } => {
                ("Inbox practice".into(), phrase.clone(), String::new())
            }
            Self::Supplied(_) => unreachable!(),
        };
        if !media.available || media.metadata != self.metadata() {
            return Err("Server did not verify the exact original reference audio".into());
        }
        Ok(LessonSubmission {
            submission_id: id,
            item_id: format!("coach-{id}"),
            expected_generation: None,
            language: Language::En,
            title,
            context,
            note: String::new(),
            phrases: vec![LessonPhraseInput {
                source_id: "phrase".into(),
                text: phrase,
                reference_audio_id: Some(media.id),
                audio_sha256: Some(media.sha256.clone()),
                expected_revision: None,
            }],
        })
    }
}
