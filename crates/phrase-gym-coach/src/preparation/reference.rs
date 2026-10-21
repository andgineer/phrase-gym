use std::sync::atomic::AtomicBool;

use phrase_gym_contracts::english::{EnglishTask, PreparedEnglishPhrase};
use phrase_gym_contracts::inbox::{JobClaim, LessonOutput};
use phrase_gym_contracts::media::MediaAsset;
use uuid::Uuid;

use crate::platform::{files, paths::CoachPaths};
use crate::transfer::{client::ServerClient, pending, text};

pub(super) fn complete(
    client: &ServerClient,
    paths: &CoachPaths,
    claim: &JobClaim,
    output: &mut LessonOutput,
    stop: &AtomicBool,
) -> Result<(), String> {
    validate_speech_work(&output.package.phrases)?;
    for phrase in &mut output.package.phrases {
        if phrase.reference_audio_id.is_nil() {
            phrase.reference_audio_id = reference(client, paths, claim, &phrase.text, stop)?;
        }
        if let Some(EnglishTask::WordSense { contexts, .. }) = &mut phrase.task {
            for context in contexts {
                context.reference_audio_id = if context.answer == phrase.text {
                    phrase.reference_audio_id
                } else {
                    reference(client, paths, claim, &context.answer, stop)?
                };
            }
        }
    }
    Ok(())
}

fn reference(
    client: &ServerClient,
    paths: &CoachPaths,
    claim: &JobClaim,
    wording: &str,
    stop: &AtomicBool,
) -> Result<Uuid, String> {
    client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
    let directory = paths
        .state
        .join(format!("reference-{}", claim.receipt.generation_id));
    std::fs::create_dir_all(&directory).map_err(|_| "Cannot retain generated reference state")?;
    let path = directory.join(format!("{}.json", files::sha256(wording.as_bytes())));
    let audio = if let Some(saved) = pending::load(&path)? {
        saved
    } else {
        let draft = text::TextDraft::new(&client.server, wording)?;
        let saved = text::synthesize(paths, &draft, stop)?;
        pending::save(&path, &saved)?;
        saved
    };
    audio.verify(&client.server, client.identity)?;
    if !matches!(&audio.material, crate::transfer::material::Material::Generated { phrase, .. } if phrase == wording)
    {
        return Err("Generated reference does not match the prepared wording".into());
    }
    client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
    let media: MediaAsset = client.post("/api/v1/media", &audio.upload)?;
    if media.sha256 != audio.audio_sha256
        || media.metadata != audio.upload.metadata
        || !media.available
    {
        return Err("Server did not verify the exact generated reference".into());
    }
    Ok(media.id)
}

fn validate_speech_work(phrases: &[PreparedEnglishPhrase]) -> Result<(), String> {
    if !(1..=5).contains(&phrases.len())
        || phrases.iter().any(|phrase| {
            phrase.text.len() > 200
                || match &phrase.task {
                    Some(EnglishTask::WordSense { contexts, .. }) => {
                        !(2..=3).contains(&contexts.len())
                            || contexts.iter().any(|context| {
                                context.answer.len() > 200 || !context.reference_audio_id.is_nil()
                            })
                    }
                    _ => false,
                }
        })
    {
        return Err("Prepared speech must contain at most five targets, two or three contexts per word, short answers and no model-supplied context audio identities".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn model_output_cannot_multiply_speech_work_before_validation() {
        let context = json!({"situation_id":"one", "answer":"Please clarify this.", "reference_audio_id":Uuid::nil()});
        let mut phrase: PreparedEnglishPhrase = serde_json::from_value(json!({"source_id":"word", "meaning_key":"make-clear", "intention_key":"make-clear", "text":"Please clarify this.", "accepted_variants":[], "situations":[], "hint":"cl", "help":null, "familiarity":{"kind":"new"}, "reference_audio_id":Uuid::nil(), "expected_revision":null, "task":{"kind":"word_sense", "word":"clarify", "sense":"make clear", "contexts":[context.clone(), context.clone()]}})).unwrap();
        validate_speech_work(&[phrase.clone()]).unwrap();
        if let Some(EnglishTask::WordSense { contexts, .. }) = &mut phrase.task {
            contexts.extend([
                serde_json::from_value(context.clone()).unwrap(),
                serde_json::from_value(context).unwrap(),
            ]);
        }
        assert!(validate_speech_work(&[phrase.clone()]).is_err());
        phrase.task = None;
        phrase.text = "x".repeat(201);
        assert!(validate_speech_work(&[phrase]).is_err());
    }
}
