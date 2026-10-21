use std::path::Path;

use phrase_gym_contracts::english::{
    EnglishPackageVersion, EnglishTask, PreparedEnglishLesson, PreparedEnglishPhrase,
};
use phrase_gym_contracts::inbox::{JobClaim, LessonOutput};
use uuid::Uuid;

use super::readiness::{MODEL, VERIFIED_VERSION};
use crate::process::Ending;

pub fn read(directory: &Path, claim: &JobClaim) -> Result<LessonOutput, String> {
    read_selected(
        directory,
        claim,
        &format!("claude-{VERIFIED_VERSION}-{MODEL}-demo-v1"),
    )
}

pub fn read_selected(
    directory: &Path,
    claim: &JobClaim,
    identity: &str,
) -> Result<LessonOutput, String> {
    let bytes = crate::platform::files::read_selected(&directory.join("stdout.log"), 64 * 1024)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(
        |_| "Selected CLI returned empty or malformed output; inspect the private attempt",
    )?;
    let phrases: Vec<PreparedEnglishPhrase> = if claim.input.phrases.len() == 1 {
        vec![
            serde_json::from_value(value)
                .map_err(|_| "Selected CLI returned unsupported target output")?,
        ]
    } else {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Batch {
            phrases: Vec<PreparedEnglishPhrase>,
        }
        serde_json::from_value::<Batch>(value)
            .map_err(|_| "Selected CLI returned unsupported batch output")?
            .phrases
    };
    if phrases.len() != claim.input.phrases.len() {
        return Err("Selected CLI omitted accepted material".into());
    }
    for (phrase, input) in phrases.iter().zip(&claim.input.phrases) {
        if !phrase_gym_contracts::english_source::matches_task(&input.text, phrase.task.as_ref()) {
            return Err("Selected CLI changed the requested fixed phrase, declared frame or word sense; nothing was synthesized or published".into());
        }
        validate_variants(phrase)?;
        validate_task_shape(phrase)?;
        if phrase.source_id != input.source_id
            || phrase.source_text.as_ref().unwrap_or(&phrase.text) != &input.text
            || phrase.reference_audio_id != input.reference_audio_id.unwrap_or(Uuid::nil())
            || phrase.expected_revision != input.expected_revision
            || ((input.reference_audio_id.is_some()
                || (!input.text.contains('{')
                    && phrase_gym_contracts::english_source::word_form(&input.text).is_none()))
                && phrase.text != input.text)
            || phrase.help.is_none()
        {
            return Err("Selected CLI output changed accepted source identity or supplied audio; nothing was published".into());
        }
    }
    Ok(LessonOutput {
        fence: claim.fence.clone(),
        publication_id: Uuid::new_v5(&claim.fence.token, identity.as_bytes()),
        adapter: identity.into(),
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
                target_source_ids: claim
                    .input
                    .phrases
                    .iter()
                    .map(|p| p.source_id.clone())
                    .collect(),
            },
            phrases,
        },
    })
}

pub fn failure(directory: &Path, ending: Ending) -> String {
    match ending {
        Ending::TimedOut => {
            return "Selected CLI preparation exceeded 90 seconds; accepted input remains on Server"
                .into();
        }
        Ending::Cancelled => {
            return "Selected CLI preparation cancelled; reconcile the accepted lesson before retrying"
                .into();
        }
        Ending::OutputLimitExceeded => {
            return "Selected CLI output exceeded 64 KiB; nothing was published".into();
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
        "Selected CLI subscription login required; sign in outside Coach, then retry preparation"
            .into()
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
        "Selected CLI quota or capacity limit; wait for the provider's stated reset or recovery, then retry. No fallback was used".into()
    } else {
        "Selected CLI preparation failed; check the private attempt and selected tool. No output was published".into()
    }
}

fn wording(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn validate_variants(phrase: &PreparedEnglishPhrase) -> Result<(), String> {
    let mut variants = std::collections::BTreeSet::from([wording(&phrase.text)]);
    if phrase
        .accepted_variants
        .iter()
        .any(|variant| !variants.insert(wording(variant)))
    {
        return Err("Selected CLI accepted variants repeat canonical or alternative wording; nothing was synthesized or published".into());
    }
    Ok(())
}

fn validate_task_shape(phrase: &PreparedEnglishPhrase) -> Result<(), String> {
    let valid = match &phrase.task {
        None => true,
        Some(EnglishTask::Frame { pattern, slots }) => {
            phrase_gym_contracts::english_source::frame_example_matches(
                pattern,
                slots,
                &phrase.text,
            )
        }
        Some(EnglishTask::WordSense { word, contexts, .. }) => {
            (2..=3).contains(&contexts.len())
                && contexts.len() == phrase.situations.len()
                && contexts
                    .first()
                    .is_some_and(|context| context.answer == phrase.text)
                && contexts.iter().all(|context| {
                    phrase
                        .situations
                        .iter()
                        .find(|situation| situation.id == context.situation_id)
                        .is_some_and(|situation| {
                            situation.partner.is_none()
                                && phrase_gym_contracts::english_source::word_gap_matches(
                                    &situation.text,
                                    &context.answer,
                                    word,
                                )
                        })
                })
        }
    };
    if valid {
        Ok(())
    } else {
        Err("Selected CLI frame example or word gaps do not match their answers; nothing was synthesized or published".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn punctuation_only_variants_fail_before_speech_without_rewriting_model_output() {
        let mut phrase: PreparedEnglishPhrase = serde_json::from_value(json!({
            "source_id":"one", "meaning_key":"repeat", "intention_key":"repeat", "text":"Could you repeat that, please?", "accepted_variants":["Could you repeat that please?"],
            "situations":[], "hint":"repeat", "help":null, "familiarity":{"kind":"new"}, "reference_audio_id":Uuid::nil(), "expected_revision":null
        })).unwrap();
        assert!(
            validate_variants(&phrase)
                .unwrap_err()
                .contains("nothing was synthesized")
        );
        assert_eq!(phrase.accepted_variants, ["Could you repeat that please?"]);
        phrase.accepted_variants = vec!["Could you say that again, please?".into()];
        validate_variants(&phrase).unwrap();
    }
}
