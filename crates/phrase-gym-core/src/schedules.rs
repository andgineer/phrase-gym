use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::CoreError;
use crate::storage::StorageError;
use crate::timestamps;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Schedule {
    pub version: i64,
    pub due_at: DateTime<Utc>,
    pub memory_state: Option<String>,
}

pub(crate) fn find(connection: &Connection, target: Uuid) -> Result<Option<Schedule>, CoreError> {
    let row: Option<(i64, String, Option<String>)> = connection
        .query_row(
            "SELECT version, due_at, memory_state FROM target_schedule WHERE target_id = ?1",
            [target.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    row.map(|(version, due_at, memory_state)| {
        let due_at = timestamps::parse(&due_at).map_err(|error| {
            StorageError::CorruptRecord(format!("due time {due_at} of {target}: {error}"))
        })?;
        Ok(Schedule {
            version,
            due_at,
            memory_state,
        })
    })
    .transpose()
}

pub(crate) fn save(
    connection: &Connection,
    target: Uuid,
    schedule: &Schedule,
    policy_version: &str,
) -> Result<(), CoreError> {
    connection.execute(
        "INSERT INTO target_schedule (target_id, version, due_at, memory_state, policy_version) \
         VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT (target_id) DO UPDATE SET \
         version = excluded.version, due_at = excluded.due_at, \
         memory_state = excluded.memory_state, policy_version = excluded.policy_version",
        params![
            target.to_string(),
            schedule.version,
            timestamps::isoformat(schedule.due_at),
            schedule.memory_state,
            policy_version,
        ],
    )?;
    Ok(())
}
