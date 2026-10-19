use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use phrase_gym_contracts::inbox::LessonSubmission;
use phrase_gym_contracts::media::MediaUpload;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::supplied::SuppliedLesson;

pub const MAX_AUDIO: u64 = 1_000_000;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingSend {
    pub server: String,
    pub submission_id: Uuid,
    pub claim_id: Uuid,
    pub supplied: SuppliedLesson,
    pub upload: MediaUpload,
    pub submission: Option<LessonSubmission>,
}

pub fn read_selected(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "Cannot read the selected file")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit {
        return Err(
            "Select a regular file within the preview size limit; symlinks are unsupported".into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "Cannot open selected file")?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read selected file")?;
    if bytes.len() as u64 > limit {
        return Err("Selected file exceeds the preview limit".into());
    }
    Ok(bytes)
}

pub fn load(path: &Path) -> Result<Option<PendingSend>, String> {
    if !path.exists() {
        return Ok(None);
    }
    serde_json::from_slice(&read_selected(path, 2_000_000)?)
        .map(Some)
        .map_err(|_| {
            "Pending Send is unreadable; preserve it and inspect the local state file".into()
        })
}

pub fn save(path: &Path, pending: &PendingSend) -> Result<(), String> {
    let bytes = serde_json::to_vec(pending).map_err(|_| "Cannot encode pending Send")?;
    if bytes.len() > 2_000_000 {
        return Err("Pending Send exceeds the preview limit".into());
    }
    let temporary = path.with_extension(format!("pending-{}.tmp", Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary).map_err(
        |_| "Cannot create protected pending Send; preserve any existing temporary file",
    )?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "Cannot persist pending Send")?;
    fs::rename(&temporary, path).map_err(|_| "Cannot commit pending Send")?;
    #[cfg(unix)]
    File::open(
        path.parent()
            .ok_or("Pending Send needs a parent directory")?,
    )
    .and_then(|f| f.sync_all())
    .map_err(|_| "Cannot sync pending Send folder")?;
    Ok(())
}
