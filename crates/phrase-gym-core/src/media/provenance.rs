use phrase_gym_contracts::media::{MediaMetadata, MediaOrigin};

use crate::content::structure::text;
use crate::error::CoreError;

fn check(metadata: &MediaMetadata) -> Result<(), String> {
    text("audio text", &metadata.text)?;
    text("audio source", &metadata.source)?;
    if metadata.locale.split('-').next() != Some(metadata.language.code()) {
        return Err("Audio locale must match the learning language".to_owned());
    }
    let present = |value: &Option<String>| value.as_deref().is_some_and(|text| !text.is_empty());
    let generated_complete =
        present(&metadata.engine) && present(&metadata.voice) && present(&metadata.speech_text);
    if metadata.origin == MediaOrigin::Generated && !generated_complete {
        return Err("Generated audio requires engine, voice and exact speech input".to_owned());
    }
    let performed = matches!(metadata.origin, MediaOrigin::Actor | MediaOrigin::Course);
    if performed && !present(&metadata.performer) {
        return Err("Actor/course audio requires performer provenance".to_owned());
    }
    Ok(())
}

pub fn validate(metadata: &MediaMetadata) -> Result<(), CoreError> {
    check(metadata).map_err(CoreError::Invalid)
}
