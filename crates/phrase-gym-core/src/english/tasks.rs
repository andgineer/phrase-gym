use std::collections::BTreeMap;

use phrase_gym_contracts::content::TargetKind;
use phrase_gym_contracts::english::{EnglishTask, PreparedEnglishPhrase};
use uuid::Uuid;

use super::structure::{text, words};
use crate::error::CoreError;

pub(crate) fn kind(phrase: &PreparedEnglishPhrase) -> TargetKind {
    match phrase.task {
        None => TargetKind::FixedPhrase,
        Some(EnglishTask::Frame { .. }) => TargetKind::Frame,
        Some(EnglishTask::WordSense { .. }) => TargetKind::WordSense,
    }
}

pub(crate) fn slots(phrase: &PreparedEnglishPhrase) -> BTreeMap<String, Vec<String>> {
    match &phrase.task {
        Some(EnglishTask::Frame { slots, .. }) => slots
            .iter()
            .map(|slot| (slot.name.clone(), slot.values.clone()))
            .collect(),
        _ => BTreeMap::new(),
    }
}

pub(crate) fn context_audio(phrase: &PreparedEnglishPhrase) -> Vec<Uuid> {
    match &phrase.task {
        Some(EnglishTask::WordSense { contexts, .. }) => {
            contexts.iter().map(|c| c.reference_audio_id).collect()
        }
        _ => Vec::new(),
    }
}

pub(crate) fn validate(phrase: &PreparedEnglishPhrase) -> Result<(), CoreError> {
    if let Some(source) = &phrase.source_text {
        text(source, "original source", 800)?;
        if !phrase_gym_contracts::english_source::matches_task(source, phrase.task.as_ref()) {
            return Err(CoreError::Invalid("Prepared task must preserve the original fixed phrase, declared frame or selected word form".into()));
        }
    }
    match &phrase.task {
        None => {}
        Some(EnglishTask::Frame { pattern, slots }) => {
            text(pattern, "frame pattern", 300)?;
            if !(1..=3).contains(&slots.len()) {
                return Err(CoreError::Invalid(
                    "A frame needs one to three named variable slots".into(),
                ));
            }
            let mut names = std::collections::BTreeSet::new();
            for slot in slots {
                let name = &slot.name;
                let values = &slot.values;
                if !names.insert(name) {
                    return Err(CoreError::Invalid(
                        "Frame slot names must be distinct".into(),
                    ));
                }
                text(name, "slot name", 40)?;
                let marker = format!("{{{name}}}");
                if !name.chars().all(|c| c.is_alphanumeric() || c == '-')
                    || pattern.matches(&marker).count() != 1
                    || !(2..=5).contains(&values.len())
                {
                    return Err(CoreError::Invalid(
                        "Every frame slot needs one named marker and two to five example values"
                            .into(),
                    ));
                }
                for value in values {
                    text(value, "slot value", 100)?;
                }
            }
            if !phrase_gym_contracts::english_source::frame_example_matches(
                pattern,
                slots,
                &phrase.text,
            ) {
                return Err(CoreError::Invalid("Frame reference must be a complete example of its declared pattern and slot values".into()));
            }
        }
        Some(EnglishTask::WordSense {
            word,
            sense,
            contexts,
        }) => {
            text(word, "word form", 100)?;
            text(sense, "word sense", 300)?;
            if words(word).len() != 1
                || !(2..=3).contains(&contexts.len())
                || contexts.len() != phrase.situations.len()
                || contexts[0].answer != phrase.text
            {
                return Err(CoreError::Invalid(
                    "A word sense needs one form and two or three matching short contexts".into(),
                ));
            }
            let mut seen = std::collections::BTreeSet::new();
            for context in contexts {
                text(&context.answer, "word context answer", 200)?;
                let situation = phrase
                    .situations
                    .iter()
                    .find(|s| s.id == context.situation_id)
                    .ok_or_else(|| {
                        CoreError::Invalid(
                            "Word context has no matching retrieval situation".into(),
                        )
                    })?;
                if !seen.insert(&context.situation_id)
                    || context.reference_audio_id.is_nil()
                    || situation.partner.is_some()
                    || situation.text.matches("_____").count() != 1
                    || !words(&context.answer).contains(&word.to_lowercase())
                {
                    return Err(CoreError::Invalid("Word contexts need distinct matching gaps, target forms and verified reference audio".into()));
                }
                if !phrase_gym_contracts::english_source::word_gap_matches(
                    &situation.text,
                    &context.answer,
                    word,
                ) {
                    return Err(CoreError::Invalid(
                        "The word-context answer must fill its exact prompt gap".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}
