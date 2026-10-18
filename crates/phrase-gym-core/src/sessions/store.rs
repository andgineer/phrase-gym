use chrono::{DateTime, Utc};
use phrase_gym_contracts::sessions::{
    PracticeSession, SessionManifest, SessionStatus, SessionStep,
};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;
use crate::storage::StorageError;
use crate::timestamps;

use super::attempts;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PendingRepair {
    pub target_index: usize,
    pub after_step: usize,
    pub target_id: Uuid,
    pub eligible_at: String,
}

// The manifest is written once at start; later updates change only the session's position.
#[derive(Clone, Debug)]
pub(crate) struct SessionRow {
    pub id: Uuid,
    pub start_payload: String,
    pub manifest: SessionManifest,
    pub status: SessionStatus,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub queue: Vec<SessionStep>,
    pub repairs: Vec<PendingRepair>,
    pub cursor: usize,
    pub last_sequence: i64,
}

impl SessionRow {
    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.version += 1;
        self.updated_at = now;
    }
}

const COLUMNS: &str = "id, start_payload, manifest, status, version, created_at, updated_at, \
                       queue, repairs, cursor, last_sequence";

type Columns = (
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
    String,
    String,
    i64,
    i64,
);

fn columns(row: &Row<'_>) -> rusqlite::Result<Columns> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
        row.get(10)?,
    ))
}

fn instant(what: &str, text: &str) -> Result<DateTime<Utc>, CoreError> {
    timestamps::parse(text)
        .map_err(|error| StorageError::CorruptRecord(format!("{what} {text}: {error}")).into())
}

fn session(
    (id, start_payload, manifest, status, version, created, updated, queue, repairs, cursor, last): Columns,
) -> Result<SessionRow, CoreError> {
    let what = format!("session {id}");
    Ok(SessionRow {
        status: SessionStatus::from_code(&status)
            .ok_or_else(|| StorageError::CorruptRecord(format!("{what} status {status}")))?,
        id: records::uuid(&id)?,
        start_payload,
        manifest: records::parse(&what, &manifest)?,
        version,
        created_at: instant(&what, &created)?,
        updated_at: instant(&what, &updated)?,
        queue: records::parse(&what, &queue)?,
        repairs: records::parse(&what, &repairs)?,
        cursor: usize::try_from(cursor)
            .map_err(|_| StorageError::CorruptRecord(format!("{what} cursor {cursor}")))?,
        last_sequence: last,
    })
}

pub(crate) fn find(connection: &Connection, id: Uuid) -> Result<Option<SessionRow>, CoreError> {
    connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM practice_session WHERE id = ?1"),
            [id.to_string()],
            columns,
        )
        .optional()?
        .map(session)
        .transpose()
}

pub(crate) fn open(connection: &Connection) -> Result<Option<SessionRow>, CoreError> {
    connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM practice_session WHERE open_slot = 1"),
            [],
            columns,
        )
        .optional()?
        .map(session)
        .transpose()
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| unreachable!("session state serializes"))
}

fn position(cursor: usize) -> i64 {
    i64::try_from(cursor).unwrap_or(i64::MAX)
}

fn open_slot(status: SessionStatus) -> Option<i64> {
    status.is_open().then_some(1)
}

pub(crate) fn insert(connection: &Connection, row: &SessionRow) -> Result<(), CoreError> {
    connection.execute(
        &format!(
            "INSERT INTO practice_session ({COLUMNS}, open_slot) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
        ),
        params![
            row.id.to_string(),
            row.start_payload,
            json(&row.manifest),
            row.status.code(),
            row.version,
            timestamps::isoformat(row.created_at),
            timestamps::isoformat(row.updated_at),
            json(&row.queue),
            json(&row.repairs),
            position(row.cursor),
            row.last_sequence,
            open_slot(row.status),
        ],
    )?;
    Ok(())
}

pub(crate) fn update(connection: &Connection, row: &SessionRow) -> Result<(), CoreError> {
    connection.execute(
        "UPDATE practice_session SET status = ?2, open_slot = ?3, version = ?4, updated_at = ?5, \
         queue = ?6, repairs = ?7, cursor = ?8, last_sequence = ?9 WHERE id = ?1",
        params![
            row.id.to_string(),
            row.status.code(),
            open_slot(row.status),
            row.version,
            timestamps::isoformat(row.updated_at),
            json(&row.queue),
            json(&row.repairs),
            position(row.cursor),
            row.last_sequence,
        ],
    )?;
    Ok(())
}

pub(crate) fn view(
    connection: &Connection,
    row: &SessionRow,
) -> Result<PracticeSession, CoreError> {
    Ok(PracticeSession {
        id: row.id,
        status: row.status,
        version: row.version,
        created_at: timestamps::isoformat(row.created_at),
        updated_at: timestamps::isoformat(row.updated_at),
        manifest: row.manifest.clone(),
        cursor: position(row.cursor),
        last_sequence: row.last_sequence,
        queue: row.queue.clone(),
        attempts: attempts::of_session(connection, row.id)?,
    })
}
