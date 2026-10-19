use std::collections::BTreeSet;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::inbox::LessonSubmission;
use phrase_gym_contracts::media::{MediaOrigin, MediaRole};
use rusqlite::Connection;

use crate::content::records;
use crate::error::CoreError;
use crate::media::index::MediaIndex;
use crate::media::store::MediaStore;
use crate::storage::Database;

pub(super) fn text(value: &str, name: &str, limit: usize) -> Result<(), CoreError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.len() > limit
        || value.chars().any(char::is_control)
    {
        return Err(CoreError::Invalid(format!(
            "{name} must be nonempty trimmed text of at most {limit} bytes"
        )));
    }
    Ok(())
}

pub(super) fn expected_revisions(
    connection: &Connection,
    input: &LessonSubmission,
    namespace: &str,
) -> Result<(), CoreError> {
    for phrase in &input.phrases {
        let existing = records::sourced_target(connection, namespace, &phrase.source_id)?;
        if existing.map(|target| target.current_revision) != phrase.expected_revision {
            return Err(CoreError::Conflict(
                "Expected target revision changed".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate(
    database: &Database,
    store: &MediaStore,
    input: &LessonSubmission,
) -> Result<(), CoreError> {
    text(&input.item_id, "Item identity", 200)?;
    text(&input.title, "Lesson title", 300)?;
    if input.submission_id.is_nil()
        || input.language != Language::En
        || !(1..=5).contains(&input.phrases.len())
        || input.context.len() > 8_000
        || input.note.len() > 2_000
        || input.expected_generation.is_some_and(|n| n < 1)
    {
        return Err(CoreError::Invalid("Submit one lesson of at most five fixed English phrases with supplied audio; words, dialogues, missing audio and bulk imports are unsupported".into()));
    }
    let index = MediaIndex::load(
        database,
        store,
        input.phrases.iter().map(|p| p.reference_audio_id),
    )?;
    let mut ids = BTreeSet::new();
    for phrase in &input.phrases {
        text(&phrase.source_id, "Phrase identity", 200)?;
        text(&phrase.text, "Fixed phrase", 200)?;
        if !ids.insert(&phrase.source_id)
            || phrase.text.split_whitespace().count() > 30
            || phrase.text.contains(['{', '}', '_'])
            || (phrase.text.starts_with("https://") || phrase.text.starts_with("http://"))
            || phrase.expected_revision.is_some_and(|n| n < 1)
        {
            return Err(CoreError::Invalid("Submit distinct short fixed phrases and extracted text; URL-only input and slots are unsupported".into()));
        }
        let media = index
            .record(phrase.reference_audio_id)
            .filter(|_| index.available(phrase.reference_audio_id))
            .ok_or_else(|| {
                CoreError::Invalid(
                    "Upload the complete original reference audio before Send".into(),
                )
            })?;
        if media.sha256 != phrase.audio_sha256
            || media.metadata.text != phrase.text
            || media.metadata.language != Language::En
            || media.metadata.role != MediaRole::Reference
            || media.metadata.origin == MediaOrigin::Generated
        {
            return Err(CoreError::Invalid(
                "Supplied audio must match its hash, exact English phrase and reference role"
                    .into(),
            ));
        }
    }
    Ok(())
}
