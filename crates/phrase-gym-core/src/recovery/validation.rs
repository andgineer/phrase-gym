use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::Connection;

use crate::error::CoreError;
use crate::hashing::sha256_hex;
use crate::storage::DATABASE_FILE;
use crate::storage::connection::open_inspector;
use crate::storage::migrations::{APPLICATION_ID, MIGRATIONS, latest_revision, recognize_file};

use super::files;
use super::manifest::Digest;

pub(super) fn database(directory: &Path) -> Result<(i64, BTreeMap<String, Digest>), CoreError> {
    let path = directory.join(DATABASE_FILE);
    files::regular(&path)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        if path
            .with_file_name(format!("{DATABASE_FILE}{suffix}"))
            .try_exists()?
        {
            return Err(CoreError::Invalid(
                "Snapshot must be a standalone SQLite database".to_owned(),
            ));
        }
    }
    recognize_file(&path, MIGRATIONS)?;
    let connection = open_inspector(&path)?;
    let application: i64 = connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    let revision: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if application != APPLICATION_ID || !(1..=latest_revision(MIGRATIONS)).contains(&revision) {
        return Err(CoreError::Invalid(
            "Snapshot has no released Rust schema".to_owned(),
        ));
    }
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    let violations: i64 =
        connection.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    if integrity != "ok" || violations != 0 {
        return Err(CoreError::Invalid(
            "Snapshot database integrity check failed".to_owned(),
        ));
    }
    if revision >= 2 {
        recordings(&connection)?;
    }
    let mut statement = connection.prepare("SELECT sha256, byte_length FROM reference_media")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let mut media = BTreeMap::new();
    for row in rows {
        let (sha256, byte_length) = row?;
        let byte_length = u64::try_from(byte_length)
            .map_err(|_| CoreError::Invalid("Invalid media length".to_owned()))?;
        if sha256.len() != 64
            || !sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(CoreError::Invalid(
                "Invalid media hash in snapshot".to_owned(),
            ));
        }
        let digest = Digest {
            sha256: sha256.clone(),
            byte_length,
        };
        if media
            .insert(sha256, digest)
            .is_some_and(|previous| previous.byte_length != byte_length)
        {
            return Err(CoreError::Invalid(
                "Conflicting media lengths in snapshot".to_owned(),
            ));
        }
    }
    Ok((revision, media))
}

fn recordings(connection: &Connection) -> Result<(), CoreError> {
    let mut statement = connection.prepare("SELECT audio, sha256 FROM learner_recording")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (audio, sha256) = row?;
        if sha256_hex(&audio) != sha256 {
            return Err(CoreError::Invalid(
                "Recording checksum mismatch in snapshot".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(super) fn media(
    directory: &Path,
    expected: &BTreeMap<String, Digest>,
) -> Result<(), CoreError> {
    for (hash, digest) in expected {
        let path = directory.join("media").join(hash);
        files::regular(&path)?;
        digest.check(&std::fs::read(path)?)?;
    }
    Ok(())
}
