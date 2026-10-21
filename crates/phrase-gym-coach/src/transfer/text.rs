use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::media::{MediaRole, MediaUpload, ReferenceMime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{material::Material, pending::PendingSend};
use crate::platform::{files, paths::CoachPaths};
use crate::speech::{PiperVoice, SpeechRequest};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextDraft {
    pub server: String,
    pub phrase: String,
    pub submission_id: Uuid,
    pub claim_id: Uuid,
}

impl TextDraft {
    pub fn new(server: &str, phrase: &str) -> Result<Self, String> {
        entries(phrase)?;
        Ok(Self {
            server: super::client::server_origin(server)?,
            phrase: phrase.into(),
            submission_id: Uuid::new_v4(),
            claim_id: Uuid::new_v4(),
        })
    }

    pub fn retain(path: &Path, server: &str, phrase: &str) -> Result<Self, String> {
        let server = super::client::server_origin(server)?;
        if path.exists() {
            let saved: Self = serde_json::from_slice(&files::read_selected(path, 8192)?)
                .map_err(|_| "Saved text input is unreadable; preserve it before repair")?;
            if saved.server != server || saved.phrase != phrase {
                return Err(
                    "A different text input is retained; retry it before adding another phrase"
                        .into(),
                );
            }
            return Ok(saved);
        }
        let draft = Self::new(&server, phrase)?;
        files::save(path, &draft)?;
        Ok(draft)
    }

    pub fn with_speech(
        &self,
        bytes: &[u8],
        metadata: phrase_gym_contracts::media::MediaMetadata,
    ) -> Result<PendingSend, String> {
        if bytes.is_empty()
            || bytes.len() as u64 > super::pending::MAX_AUDIO
            || metadata.text != self.phrase
            || metadata.language != Language::En
            || metadata.role != MediaRole::Reference
            || metadata.origin != phrase_gym_contracts::media::MediaOrigin::Generated
        {
            return Err("Generated speech does not match the retained English text".into());
        }
        Ok(PendingSend {
            server: self.server.clone(),
            identity: None,
            failure: None,
            audio_sha256: files::sha256(bytes),
            output: None,
            submission_id: self.submission_id,
            claim_id: self.claim_id,
            material: Material::Generated {
                phrase: self.phrase.clone(),
                metadata: metadata.clone(),
            },
            upload: MediaUpload {
                metadata,
                mime_type: ReferenceMime::Wav,
                audio_base64: STANDARD.encode(bytes),
            },
            submission: None,
        })
    }
}

pub fn synthesize(
    paths: &CoachPaths,
    draft: &TextDraft,
    stop: &AtomicBool,
) -> Result<PendingSend, String> {
    let configuration = files::read_selected(&paths.config.join("piper-voice.json"), 16_384)
        .map_err(|_| "Configure a licensed English Piper voice in Coach's piper-voice.json; see docs/coach-desktop.md")?;
    let voice: PiperVoice = serde_json::from_slice(&configuration).map_err(
        |_| "Coach's piper-voice.json is invalid; check the documented voice configuration",
    )?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Cannot initialize speech supervision")?;
    runtime.block_on(async {
        let request = SpeechRequest {
            language: Language::En,
            text: draft.phrase.clone(),
            speech_text: draft.phrase.clone(),
            source: "coach-deliberate-text".into(),
            role: MediaRole::Reference,
        };
        let speech = crate::speech::stage(&paths.cache, &voice, &request).await?;
        let bytes = speech
            .execute(async {
                while !stop.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            })
            .await?;
        draft.with_speech(&bytes, speech.metadata)
    })
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptedText {
    draft: TextDraft,
    receipt: phrase_gym_contracts::inbox::LessonReceipt,
}

pub fn retry_preparation(
    paths: &CoachPaths,
    client: &super::client::ServerClient,
    phrase: Option<&str>,
) -> Result<(), String> {
    let accepted: AcceptedText = serde_json::from_slice(&files::read_selected(
        &paths.state.join("text-receipt.json"),
        8192,
    )?)
    .map_err(|_| "No readable accepted text receipt; use the browser status panel")?;
    if accepted.draft.server != client.server
        || phrase.is_some_and(|phrase| phrase != accepted.draft.phrase)
    {
        return Err(
            "Restore the accepted text's original Server and exact phrase before retrying".into(),
        );
    }
    client.post::<_, phrase_gym_contracts::inbox::LessonStatus>(
        &format!(
            "/api/v1/inbox/lessons/{}/retry",
            accepted.receipt.generation_id
        ),
        &(),
    )?;
    Ok(())
}

pub fn submit_raw(
    paths: &CoachPaths,
    server: &str,
    phrase: &str,
    client: &super::client::ServerClient,
) -> Result<phrase_gym_contracts::inbox::LessonReceipt, String> {
    if client.server != super::client::server_origin(server)? {
        return Err("Restore the original text Server connection before sending".into());
    }
    let path = paths.state.join("pending-text.json");
    let draft = TextDraft::retain(&path, server, phrase)?;
    let accepted_path = paths.state.join("text-receipt.json");
    if accepted_path.exists() {
        let accepted: AcceptedText =
            serde_json::from_slice(&files::read_selected(&accepted_path, 8192)?)
                .map_err(|_| "Saved receipt is unreadable; open Settings for recovery")?;
        if accepted.draft.server == client.server && accepted.draft.phrase == phrase {
            let status: phrase_gym_contracts::inbox::LessonStatus = client.get(&format!(
                "/api/v1/inbox/lessons/{}",
                accepted.receipt.generation_id
            ))?;
            if status.status == phrase_gym_contracts::inbox::DeliveryStatus::Failed {
                retry_preparation(paths, client, Some(phrase))?;
                std::fs::remove_file(&path)
                    .map_err(|_| "Retry accepted; cannot clear retained local text")?;
                return Ok(accepted.receipt);
            }
        }
    }
    let input = phrase_gym_contracts::inbox::LessonSubmission {
        submission_id: draft.submission_id,
        item_id: format!("coach-{}", draft.submission_id),
        expected_generation: None,
        language: Language::En,
        title: "Inbox practice".into(),
        context: draft.phrase.clone(),
        note: String::new(),
        phrases: entries(&draft.phrase)?
            .into_iter()
            .enumerate()
            .map(
                |(index, text)| phrase_gym_contracts::inbox::LessonPhraseInput {
                    source_id: if draft.phrase.lines().count() == 1 {
                        "phrase".into()
                    } else {
                        format!("item-{}", index + 1)
                    },
                    text: text.into(),
                    reference_audio_id: None,
                    audio_sha256: None,
                    expected_revision: None,
                },
            )
            .collect(),
    };
    let receipt: phrase_gym_contracts::inbox::LessonReceipt =
        client.post("/api/v1/inbox/lessons", &input)?;
    files::save(
        &paths.state.join("text-receipt.json"),
        &AcceptedText {
            draft,
            receipt: receipt.clone(),
        },
    )?;
    // The Server now owns the exact original text; local removal follows its durable receipt.
    std::fs::remove_file(path).map_err(|_| "Accepted by Server; retry to reconcile local text")?;
    Ok(receipt)
}

pub fn entries(text: &str) -> Result<Vec<&str>, String> {
    let entries: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if !(1..=5).contains(&entries.len())
        || text.len() > 4000
        || entries.iter().any(|entry| {
            entry.len() > 800
                || entry.chars().any(char::is_control)
                || entry.contains("http://")
                || entry.contains("https://")
        })
    {
        return Err("Add up to five short words or phrases at a time, without URLs".into());
    }
    Ok(entries)
}
