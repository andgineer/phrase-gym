use phrase_gym_contracts::media::{MediaAsset, MediaMetadata};
use rusqlite::{Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::error::CoreError;
use crate::storage::StorageError;

use super::store::MediaStore;

#[derive(Clone, Debug)]
pub struct MediaRecord {
    pub id: Uuid,
    pub sha256: String,
    pub byte_length: i64,
    pub mime_type: String,
    pub metadata: MediaMetadata,
}

impl MediaRecord {
    pub fn asset(self, available: bool) -> MediaAsset {
        MediaAsset {
            id: self.id,
            sha256: self.sha256,
            byte_length: self.byte_length,
            mime_type: self.mime_type,
            metadata: self.metadata,
            available,
        }
    }

    pub fn available_in(&self, store: &MediaStore) -> bool {
        store.available(&self.sha256, self.byte_length)
    }
}

const COLUMNS: &str = "id, sha256, byte_length, mime_type, metadata";

fn decode(row: &Row<'_>) -> rusqlite::Result<(String, String, i64, String, String)> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn record(
    (id, sha256, byte_length, mime_type, metadata): (String, String, i64, String, String),
) -> Result<MediaRecord, CoreError> {
    let corrupt =
        |detail: String| StorageError::CorruptRecord(format!("reference media {id}: {detail}"));
    Ok(MediaRecord {
        id: Uuid::parse_str(&id).map_err(|error| corrupt(error.to_string()))?,
        metadata: serde_json::from_str(&metadata).map_err(|error| corrupt(error.to_string()))?,
        sha256,
        byte_length,
        mime_type,
    })
}

pub fn find(connection: &Connection, id: Uuid) -> Result<Option<MediaRecord>, CoreError> {
    connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM reference_media WHERE id = ?1"),
            [id.to_string()],
            decode,
        )
        .optional()?
        .map(record)
        .transpose()
}

pub fn all(connection: &Connection) -> Result<Vec<MediaRecord>, CoreError> {
    let mut statement = connection.prepare(&format!(
        "SELECT {COLUMNS} FROM reference_media ORDER BY id"
    ))?;
    let rows = statement.query_map([], decode)?;
    rows.map(|row| record(row?)).collect()
}
