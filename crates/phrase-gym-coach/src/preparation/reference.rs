use std::sync::atomic::AtomicBool;

use phrase_gym_contracts::inbox::{JobClaim, LessonSubmission};
use phrase_gym_contracts::media::MediaAsset;
use serde::{Deserialize, Serialize};

use crate::platform::{files, paths::CoachPaths};
use crate::transfer::{client::ServerClient, pending, text};

#[derive(Serialize, Deserialize)]
struct Retained {
    original: LessonSubmission,
    input: LessonSubmission,
}

pub(super) fn complete(
    client: &ServerClient,
    paths: &CoachPaths,
    claim: &JobClaim,
    stop: &AtomicBool,
) -> Result<JobClaim, String> {
    if claim
        .input
        .phrases
        .iter()
        .all(|p| p.reference_audio_id.is_some())
    {
        return Ok(claim.clone());
    }
    let directory = paths
        .state
        .join(format!("reference-{}", claim.receipt.generation_id));
    std::fs::create_dir_all(&directory).map_err(|_| "Cannot retain generated reference state")?;
    let path = directory.join("input.json");
    if path.exists() {
        let saved: Retained = serde_json::from_slice(&files::read_selected(&path, 100_000)?)
            .map_err(
                |_| "Generated reference checkpoint is unreadable; preserve it before repair",
            )?;
        if saved.original != claim.input {
            return Err(
                "Generated reference checkpoint belongs to different original input".into(),
            );
        }
        let mut normalized = claim.clone();
        normalized.input = saved.input;
        return Ok(normalized);
    }
    let mut normalized = claim.clone();
    for phrase in &mut normalized.input.phrases {
        if phrase.reference_audio_id.is_some() {
            continue;
        }
        client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
        let audio_path = directory.join(format!(
            "{}.json",
            uuid::Uuid::new_v5(&claim.receipt.generation_id, phrase.source_id.as_bytes())
        ));
        let audio = if let Some(saved) = pending::load(&audio_path)? {
            saved
        } else {
            let draft = text::TextDraft::new(&client.server, &phrase.text)?;
            let saved = text::synthesize(paths, &draft, stop)?;
            pending::save(&audio_path, &saved)?;
            saved
        };
        audio.verify(&client.server, client.identity)?;
        if !matches!(&audio.material, crate::transfer::material::Material::Generated { phrase: text, .. } if text == &phrase.text)
        {
            return Err("Generated reference does not match the original phrase".into());
        }
        client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
        let media: MediaAsset = client.post("/api/v1/media", &audio.upload)?;
        if media.sha256 != audio.audio_sha256
            || media.metadata != audio.upload.metadata
            || !media.available
        {
            return Err("Server did not verify the exact generated reference".into());
        }
        phrase.reference_audio_id = Some(media.id);
        phrase.audio_sha256 = Some(media.sha256);
    }
    files::save(
        &path,
        &Retained {
            original: claim.input.clone(),
            input: normalized.input.clone(),
        },
    )?;
    Ok(normalized)
}
