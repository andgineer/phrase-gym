use chrono::SubsecRound;
use phrase_gym_contracts::practice::PracticeKind;
use phrase_gym_contracts::recordings::RecordingInput;
use rusqlite::{Connection, params};

use crate::error::CoreError;
use crate::exposures;
use crate::hashing::{canonical_json, sha256_hex};
use crate::practice::events;
use crate::sessions::attempts;
use crate::timestamps;
use crate::transfer::decode_base64;

const MAX_TAKE_BYTES: usize = 12 * 1024 * 1024;
const MAX_BASE64_CHARACTERS: usize = 17_000_000;
const MAX_MIME_CHARACTERS: usize = 100;
const ALLOWED_MIME_TYPES: [&str; 5] = [
    "audio/webm",
    "audio/ogg",
    "audio/mp4",
    "audio/wav",
    "audio/mpeg",
];

pub(crate) struct Take {
    pub audio: Vec<u8>,
    pub sha256: String,
    pub metadata: String,
}

fn invalid<T>(detail: &str) -> Result<T, CoreError> {
    Err(CoreError::Invalid(detail.to_owned()))
}

// Learner bytes stay opaque: only the transfer encoding, size and declared MIME essence are checked;
// the audio is never decoded or inspected.
pub(crate) fn take(input: &RecordingInput) -> Result<Take, CoreError> {
    if input.revision < 1 || input.duration_ms.is_some_and(|duration| duration < 0) {
        return invalid("revision must be at least 1 and duration_ms at least 0");
    }
    let mime_length = input.mime_type.chars().count();
    if mime_length == 0 || mime_length > MAX_MIME_CHARACTERS {
        return invalid("mime_type must be 1-100 characters");
    }
    if input.attempt_id.is_none() == input.exposure_id.is_none() {
        return invalid("A take links to exactly one attempt or lookup exposure");
    }
    if input.audio_base64.is_empty() || input.audio_base64.len() > MAX_BASE64_CHARACTERS {
        return invalid("audio_base64 must be 1-17000000 characters");
    }
    let captured_at = timestamps::parse(&input.captured_at)
        .map_err(|_| {
            CoreError::Invalid("captured_at must be a date-time with a time zone".to_owned())
        })?
        .trunc_subsecs(6);
    let Some(audio) = decode_base64(&input.audio_base64) else {
        return invalid("Take audio must be valid base64");
    };
    if audio.is_empty() || audio.len() > MAX_TAKE_BYTES {
        return invalid("A take must contain audio and be at most 12 MiB");
    }
    let essence = input.mime_type.split(';').next().unwrap_or_default();
    if !ALLOWED_MIME_TYPES.contains(&essence.trim().to_ascii_lowercase().as_str()) {
        return invalid("Unsupported recording MIME type");
    }
    let mut metadata = serde_json::to_value(input)
        .unwrap_or_else(|_| unreachable!("recording contracts always serialize"));
    if let Some(fields) = metadata.as_object_mut() {
        fields.remove("audio_base64");
        fields.insert(
            "captured_at".to_owned(),
            timestamps::isoformat(captured_at).into(),
        );
    }
    Ok(Take {
        sha256: sha256_hex(&audio),
        audio,
        metadata: canonical_json(&metadata),
    })
}

// A take must belong to the persisted attempt or lookup exposure it names, for the same revision.
pub(crate) fn context(connection: &Connection, input: &RecordingInput) -> Result<(), CoreError> {
    let revision_known: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM target_revision WHERE target_id = ?1 AND revision = ?2)",
        params![input.target_id.to_string(), input.revision],
        |row| row.get(0),
    )?;
    if !revision_known {
        return invalid("Take content revision not found");
    }
    if let Some(attempt_id) = input.attempt_id {
        let attempt = attempts::find(connection, attempt_id)?
            .ok_or_else(|| CoreError::Conflict("Persist the attempt before its take".to_owned()))?;
        if attempt.target_id != input.target_id || attempt.revision != input.revision {
            return invalid("Take does not match its attempt");
        }
    }
    if let Some(event_id) = input.capture_event_id {
        let respond = PracticeKind::Respond.code();
        let Some((_, attempt)) =
            events::binding(connection, event_id)?.filter(|(kind, _)| kind == respond)
        else {
            return Err(CoreError::Conflict(
                "Take requires an acknowledged capture event".to_owned(),
            ));
        };
        if Some(attempt) != input.attempt_id {
            return invalid("Capture event does not match its take");
        }
    }
    if let Some(exposure_id) = input.exposure_id {
        let bound = exposures::binding(connection, exposure_id)?;
        if bound != Some((input.target_id, input.revision)) {
            return invalid("Take does not match its lookup exposure");
        }
    }
    Ok(())
}
