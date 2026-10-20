use std::path::Path;

use phrase_gym_contracts::english::{
    EnglishPackageVersion, PreparedEnglishLesson, PreparedEnglishPhrase,
};
use phrase_gym_contracts::inbox::{JobClaim, LessonOutput};
use uuid::Uuid;

use super::readiness::{MODEL, VERIFIED_VERSION};
use crate::process::Ending;

pub fn read(directory: &Path, claim: &JobClaim) -> Result<LessonOutput, String> {
    let bytes = crate::platform::files::read_selected(&directory.join("stdout.log"), 64 * 1024)?;
    let phrase: PreparedEnglishPhrase = serde_json::from_slice(&bytes).map_err(|_| "Claude returned empty, malformed or unsupported output; inspect the private attempt before retrying preparation")?;
    let [input] = claim.input.phrases.as_slice() else {
        return Err("Expected one accepted phrase".into());
    };
    if phrase.source_id != input.source_id
        || phrase.text != input.text
        || phrase.reference_audio_id != input.reference_audio_id
        || phrase.expected_revision != input.expected_revision
    {
        return Err("Claude output changed the accepted phrase identity or supplied audio; nothing was published".into());
    }
    Ok(LessonOutput {
        fence: claim.fence.clone(),
        publication_id: Uuid::new_v5(&claim.fence.token, b"claude-english-demo-v1"),
        adapter: format!("claude-{VERIFIED_VERSION}-{MODEL}-demo-v1"),
        package: PreparedEnglishLesson {
            schema_version: EnglishPackageVersion::EnglishPhrasesV1,
            namespace: claim.receipt.namespace.clone(),
            source_revision: claim.receipt.generation.to_string(),
            language: claim.input.language,
            profile_revision: claim.receipt.profile_revision,
            guide_version: claim.receipt.guide_version.clone(),
            assessment_version: "situation-v1".into(),
            lesson: phrase_gym_contracts::content::LessonInput {
                source_id: claim.input.item_id.clone(),
                title: claim.input.title.clone(),
                target_source_ids: vec![input.source_id.clone()],
            },
            phrases: vec![phrase],
        },
    })
}

pub fn failure(directory: &Path, ending: Ending) -> String {
    match ending {
        Ending::TimedOut => {
            return "Claude preparation exceeded 90 seconds; accepted input remains on Server"
                .into();
        }
        Ending::Cancelled => {
            return "Claude preparation cancelled; reconcile the accepted lesson before retrying"
                .into();
        }
        Ending::OutputLimitExceeded => {
            return "Claude output exceeded 64 KiB; nothing was published".into();
        }
        Ending::Exited(_) => {}
    }
    let mut output = String::new();
    for stream in ["stdout.log", "stderr.log"] {
        if let Ok(bytes) = crate::platform::files::read_selected(&directory.join(stream), 64 * 1024)
        {
            output.push_str(&String::from_utf8_lossy(&bytes).to_lowercase());
        }
    }
    if output.contains("not logged in")
        || output.contains("unauthorized")
        || output.contains("authentication")
    {
        "Claude subscription login required; sign in outside Coach, then retry preparation".into()
    } else if [
        "rate limit",
        "rate_limit",
        "too many requests",
        "credit balance",
        "quota",
        "overloaded",
        "upgrade your subscription",
    ]
    .iter()
    .any(|p| output.contains(p))
    {
        "Claude quota or capacity limit; wait for the provider's stated reset or recovery, then retry. No fallback was used".into()
    } else {
        "Claude preparation failed; check the private attempt and selected tool. No output was published".into()
    }
}
