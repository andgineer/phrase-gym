use std::path::Path;

use phrase_gym_contracts::english::{
    EnglishPackageVersion, EnglishTask, PreparedEnglishLesson, PreparedEnglishPhrase,
};
use phrase_gym_contracts::inbox::{JobClaim, LessonOutput};
use uuid::Uuid;

use crate::adapter::CliAdapter;
use crate::process::Ending;

pub fn read_selected(
    directory: &Path,
    claim: &JobClaim,
    adapter: CliAdapter,
    identity: &str,
) -> Result<LessonOutput, String> {
    let bytes = crate::platform::files::read_selected(&directory.join("stdout.log"), 64 * 1024)?;
    let value = adapter.answer(&bytes)?;
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
    // Models may list items in any order; publication keeps the submitted order.
    let mut remaining = phrases;
    let mut phrases = Vec::with_capacity(remaining.len());
    for input in &claim.input.phrases {
        let index = remaining
            .iter()
            .position(|phrase| phrase.source_id == input.source_id)
            .ok_or("Selected CLI omitted accepted material")?;
        phrases.push(remaining.swap_remove(index));
    }
    for (phrase, input) in phrases.iter().zip(&claim.input.phrases) {
        if !phrase_gym_contracts::english_source::matches_task(&input.text, phrase.task.as_ref()) {
            return Err("Selected CLI changed the requested fixed phrase, declared frame or word sense; nothing was synthesized or published".into());
        }
        validate_exposures(phrase, claim)?;
        validate_help_layers(phrase, claim)?;
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
        "usage limit",
        "session limit",
        // Claude's JSON result envelope; a bare 429 would also match process and trace ids.
        "\"api_error_status\":429",
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

fn validate_exposures(phrase: &PreparedEnglishPhrase, claim: &JobClaim) -> Result<(), String> {
    if phrase.situations.iter().any(|situation| {
        situation.exposed_target_source_ids.iter().any(|source| {
            source == &phrase.source_id
                || !claim
                    .input
                    .phrases
                    .iter()
                    .any(|input| &input.source_id == source)
        })
    }) {
        return Err(
            "Selected CLI declared invalid target exposure; nothing was synthesized or published"
                .into(),
        );
    }
    Ok(())
}

fn validate_help_layers(phrase: &PreparedEnglishPhrase, claim: &JobClaim) -> Result<(), String> {
    let profile = &claim.profile.profile;
    let help = profile
        .languages
        .get(&claim.input.language)
        .and_then(|preferences| preferences.help_language);
    let short = phrase.help.as_ref().map(|help| help.language);
    let details = phrase.details.as_ref().map(|details| details.language);
    if short.is_none() || short != help || details != Some(profile.native_language) {
        return Err("Selected CLI omitted the short explanation or the native-language details, or used the wrong language; nothing was synthesized or published".into());
    }
    Ok(())
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
#[path = "result_tests.rs"]
mod tests;
