use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::Digest;
use std::fmt::Write as FormatWrite;
use std::path::Path;

use phrase_gym_contracts::inbox::LessonSubmission;
use phrase_gym_contracts::media::MediaUpload;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::material::Material;

pub const MAX_AUDIO: u64 = 1_000_000;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingSend {
    pub server: String,
    pub identity: Option<Uuid>,
    pub failure: Option<String>,
    pub audio_sha256: String,
    pub output: Option<phrase_gym_contracts::inbox::LessonOutput>,
    pub submission_id: Uuid,
    pub claim_id: Uuid,
    pub material: Material,
    pub upload: MediaUpload,
    pub submission: Option<LessonSubmission>,
}

pub fn load(path: &Path) -> Result<Option<PendingSend>, String> {
    if !path.exists() {
        return Ok(None);
    }
    serde_json::from_slice(&crate::platform::files::read_selected(path, 2_000_000)?)
        .map(Some)
        .map_err(|_| {
            "Pending Send is unreadable; preserve it and inspect the local state file".into()
        })
}

pub fn save(path: &Path, pending: &PendingSend) -> Result<(), String> {
    crate::platform::files::save(path, pending)
}

pub fn audio_hash(bytes: &[u8]) -> String {
    sha2::Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

impl PendingSend {
    pub fn verify(&self, server: &str, identity: Uuid) -> Result<(), String> {
        if self.server != server {
            return Err("Connect to the pending Send's original Server before Retry".into());
        }
        if self.identity.is_some_and(|saved| saved != identity) {
            return Err("Pending input belongs to the original Server account; restore its connection before retrying".into());
        }
        let original = STANDARD.decode(&self.upload.audio_base64).map_err(
            |_| "Retained original audio encoding is invalid; preserve the pending input",
        )?;
        if original.len() as u64 > MAX_AUDIO || audio_hash(&original) != self.audio_sha256 {
            return Err(
            "Retained original audio hash does not match; preserve the pending input before repair"
                .into(),
        );
        }
        Ok(())
    }
}
