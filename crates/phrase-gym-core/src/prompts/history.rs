use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::error::CoreError;
use crate::storage::StorageError;
use crate::timestamps;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChunkHistory {
    pub presentations: i64,
    pub recalls: i64,
    pub failures: i64,
    pub last_presented: Option<DateTime<Utc>>,
}

pub(crate) fn load(
    connection: &Connection,
    target: Uuid,
) -> Result<HashMap<String, ChunkHistory>, CoreError> {
    let mut statement = connection.prepare(
        "SELECT chunk_id, presentations, recalls, failures, last_presented \
         FROM chunk_history WHERE target_id = ?1",
    )?;
    let rows: Vec<(String, i64, i64, i64, String)> = statement
        .query_map([target.to_string()], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(chunk, presentations, recalls, failures, last)| {
            let last_presented = timestamps::parse(&last).map_err(|error| {
                StorageError::CorruptRecord(format!("chunk history {chunk}: {error}"))
            })?;
            Ok((
                chunk,
                ChunkHistory {
                    presentations,
                    recalls,
                    failures,
                    last_presented: Some(last_presented),
                },
            ))
        })
        .collect()
}

pub(crate) fn record_presentation(
    connection: &Connection,
    target: Uuid,
    chunks: &[String],
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    for chunk in chunks {
        connection.execute(
            "INSERT INTO chunk_history \
             (target_id, chunk_id, presentations, recalls, failures, last_presented) \
             VALUES (?1, ?2, 1, 0, 0, ?3) ON CONFLICT (target_id, chunk_id) DO UPDATE SET \
             presentations = presentations + 1, last_presented = excluded.last_presented",
            params![target.to_string(), chunk, timestamps::isoformat(now)],
        )?;
    }
    Ok(())
}

pub(crate) fn record_outcome(
    connection: &Connection,
    target: Uuid,
    chunks: &[String],
    recalled: bool,
) -> Result<(), CoreError> {
    for chunk in chunks {
        connection.execute(
            "UPDATE chunk_history SET recalls = recalls + ?3, failures = failures + ?4 \
             WHERE target_id = ?1 AND chunk_id = ?2",
            params![
                target.to_string(),
                chunk,
                i64::from(recalled),
                i64::from(!recalled)
            ],
        )?;
    }
    Ok(())
}
