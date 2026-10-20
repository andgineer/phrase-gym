use sha2::Digest;
use std::fmt::Write as FormatWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use serde::Serialize;
use uuid::Uuid;

pub fn read_selected(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "Cannot read the selected file")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit {
        return Err(
            "Select a regular file within the preview size limit; symlinks are unsupported".into(),
        );
    }
    let mut bytes = Vec::new();
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(
        i32::try_from(rustix::fs::OFlags::NOFOLLOW.bits())
            .map_err(|_| "Cannot protect selected-file access")?,
    );
    #[cfg(windows)]
    std::os::windows::fs::OpenOptionsExt::custom_flags(&mut options, 0x0020_0000);
    let file = options
        .open(path)
        .map_err(|_| "Cannot open selected file without following links")?;
    #[cfg(windows)]
    if std::os::windows::fs::MetadataExt::file_attributes(
        &file
            .metadata()
            .map_err(|_| "Cannot inspect selected file")?,
    ) & 0x400
        != 0
    {
        return Err("Selected reparse points and symlinks are unsupported".into());
    }
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read selected file")?;
    if bytes.len() as u64 > limit {
        return Err("Selected file exceeds the preview limit".into());
    }
    Ok(bytes)
}

pub fn save(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "Cannot encode protected local state")?;
    if bytes.len() > 2_000_000 {
        return Err("Protected local state exceeds the preview limit".into());
    }
    let temporary = path.with_extension(format!("pending-{}.tmp", Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary).map_err(
        |_| "Cannot create protected protected local state; preserve any existing temporary file",
    )?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "Cannot persist protected local state")?;
    fs::rename(&temporary, path).map_err(|_| "Cannot commit protected local state")?;
    #[cfg(unix)]
    File::open(
        path.parent()
            .ok_or("Protected local state needs a parent directory")?,
    )
    .and_then(|f| f.sync_all())
    .map_err(|_| "Cannot sync protected local state folder")?;
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> String {
    sha2::Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}
