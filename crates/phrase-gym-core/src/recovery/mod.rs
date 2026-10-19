mod files;
pub mod manifest;
mod validation;

use std::fs::File;
use std::path::Path;

use rusqlite::Connection;

use crate::error::CoreError;
use crate::storage::connection::open_inspector;
use crate::storage::migrations::{MIGRATIONS, recognize_file};
use crate::storage::{DATABASE_FILE, RECOVERY_PENDING, SNAPSHOT_MANIFEST};

use manifest::{Digest, FORMAT, Snapshot};

pub fn backup(source: &Path, destination: &Path) -> Result<Snapshot, CoreError> {
    if source.join(RECOVERY_PENDING).try_exists()? || source.join(SNAPSHOT_MANIFEST).try_exists()? {
        return Err(CoreError::Invalid(
            "Backup source must be a completed live data directory".to_owned(),
        ));
    }
    let path = source.join(DATABASE_FILE);
    files::regular(&path)?;
    recognize_file(&path, MIGRATIONS)?;
    let source_connection = open_inspector(&path)?;
    let destination = files::reserve(source, destination)?;
    let output = destination.join(DATABASE_FILE);
    // SQLite includes committed WAL pages. The media catalog is read from that exact resulting image.
    source_connection.backup(rusqlite::MAIN_DB, &output, None)?;
    let connection = Connection::open(&output)?;
    connection.pragma_update_and_check(None, "journal_mode", "DELETE", |row| {
        row.get::<_, String>(0)
    })?;
    drop(connection);
    File::options().write(true).open(&output)?.sync_all()?;
    let (schema_revision, media) = validation::database(&destination)?;
    copy_media(source, &destination, &media)?;
    let snapshot = Snapshot {
        format: FORMAT,
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        schema_revision,
        database: Digest::file(&output)?,
        media,
    };
    let encoded = serde_json::to_vec_pretty(&snapshot).map_err(|error| invalid_manifest(&error))?;
    files::write(&destination.join(SNAPSHOT_MANIFEST), &encoded)?;
    files::finish(&destination)?;
    Ok(snapshot)
}

pub fn verify(snapshot: &Path) -> Result<Snapshot, CoreError> {
    if snapshot.join(RECOVERY_PENDING).try_exists()? {
        return Err(CoreError::Invalid(
            "Recovery directory is incomplete".to_owned(),
        ));
    }
    let path = snapshot.join(SNAPSHOT_MANIFEST);
    files::regular(&path)?;
    let manifest: Snapshot =
        serde_json::from_slice(&std::fs::read(path)?).map_err(|error| invalid_manifest(&error))?;
    if manifest.format != FORMAT || manifest.app_version.is_empty() {
        return Err(CoreError::Invalid(
            "Unsupported recovery manifest".to_owned(),
        ));
    }
    validate(snapshot, &manifest)?;
    Ok(manifest)
}

pub fn restore(snapshot: &Path, destination: &Path) -> Result<Snapshot, CoreError> {
    let manifest = verify(snapshot)?;
    let destination = files::reserve(snapshot, destination)?;
    let output = destination.join(DATABASE_FILE);
    std::fs::copy(snapshot.join(DATABASE_FILE), &output)?;
    File::options().write(true).open(output)?.sync_all()?;
    copy_media(snapshot, &destination, &manifest.media)?;
    validate(&destination, &manifest)?;
    if manifest.schema_revision >= 4 {
        let connection = Connection::open(destination.join(DATABASE_FILE))?;
        crate::inbox::recovery::invalidate_leases(&connection)?;
        drop(connection);
        File::options()
            .write(true)
            .open(destination.join(DATABASE_FILE))?
            .sync_all()?;
    }
    files::finish(&destination)?;
    Ok(manifest)
}

fn validate(directory: &Path, manifest: &Snapshot) -> Result<(), CoreError> {
    files::regular(&directory.join(DATABASE_FILE))?;
    if Digest::file(&directory.join(DATABASE_FILE))? != manifest.database {
        return Err(CoreError::Invalid(
            "Snapshot database checksum or length mismatch".to_owned(),
        ));
    }
    let (revision, media) = validation::database(directory)?;
    if revision != manifest.schema_revision || media != manifest.media {
        return Err(CoreError::Invalid(
            "Manifest does not match snapshot database".to_owned(),
        ));
    }
    validation::media(directory, &media)
}

fn copy_media(
    source: &Path,
    destination: &Path,
    media: &std::collections::BTreeMap<String, Digest>,
) -> Result<(), CoreError> {
    let output = destination.join("media");
    std::fs::create_dir(&output)?;
    for (hash, digest) in media {
        let path = source.join("media").join(hash);
        files::regular(&path)?;
        let bytes = std::fs::read(path)?;
        digest.check(&bytes)?;
        files::write(&output.join(hash), &bytes)?;
    }
    files::sync(&output)?;
    Ok(())
}

fn invalid_manifest(error: &serde_json::Error) -> CoreError {
    CoreError::Invalid(format!("Invalid recovery manifest: {error}"))
}
