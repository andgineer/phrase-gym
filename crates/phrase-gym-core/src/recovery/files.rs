#[cfg(unix)]
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::CoreError;
use crate::storage::{RECOVERY_PENDING, recovery_reservation};

pub(super) fn reserve(source: &Path, destination: &Path) -> Result<PathBuf, CoreError> {
    let source = source.canonicalize()?;
    let name = destination.file_name().ok_or_else(|| {
        CoreError::Invalid("Recovery destination must name a new directory".to_owned())
    })?;
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()?;
    let destination = parent.join(name);
    if destination.starts_with(&source) {
        return Err(CoreError::Invalid(
            "Recovery destination must be outside the source".to_owned(),
        ));
    }
    if destination.try_exists()? {
        return Err(CoreError::Invalid(
            "Recovery destination already exists".to_owned(),
        ));
    }
    // A durable sibling reservation closes the crash window before the directory's marker exists.
    let reservation = recovery_reservation(&destination);
    write(&reservation, b"Incomplete recovery reservation\n")?;
    sync(&parent)?;
    std::fs::create_dir(&destination)?;
    write(
        &destination.join(RECOVERY_PENDING),
        b"Incomplete recovery; do not serve this directory.\n",
    )?;
    sync(&destination)?;
    std::fs::remove_file(reservation)?;
    sync(&parent)?;
    Ok(destination)
}

pub(super) fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

pub(super) fn finish(directory: &Path) -> std::io::Result<()> {
    sync(directory)?;
    std::fs::remove_file(directory.join(RECOVERY_PENDING))?;
    sync(directory)
}

#[cfg(unix)]
pub(super) fn sync(directory: &Path) -> std::io::Result<()> {
    File::open(directory)?.sync_all()
}

#[cfg(not(unix))]
pub(super) fn sync(_: &Path) -> std::io::Result<()> {
    Ok(())
}

pub(super) fn regular(path: &Path) -> Result<(), CoreError> {
    if !std::fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(CoreError::Invalid(format!(
            "Expected a regular recovery file: {}",
            path.display()
        )));
    }
    Ok(())
}
