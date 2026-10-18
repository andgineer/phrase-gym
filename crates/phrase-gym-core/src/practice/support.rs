use chrono::{DateTime, TimeDelta, Utc};
use rusqlite::Connection;
use uuid::Uuid;

use crate::error::CoreError;
use crate::storage::StorageError;
use crate::timestamps;

const RECENT: TimeDelta = TimeDelta::minutes(5);

// Help seen shortly before counts as support: lookup exposures when received, a dialogue script
// when shown, and answer, explanation, reference or comparison on any attempt at the target.
pub(crate) fn recent(
    connection: &Connection,
    target: Uuid,
    now: DateTime<Utc>,
) -> Result<bool, CoreError> {
    let mut statement = connection.prepare(
        "SELECT received_at FROM exposure WHERE target_id = ?1 \
         UNION ALL SELECT occurred_at FROM dialogue_exposure WHERE target_id = ?1 \
         UNION ALL SELECT e.occurred_at FROM practice_event e \
         JOIN practice_attempt a ON a.id = e.attempt_id \
         WHERE a.target_id = ?1 \
         AND e.kind IN ('answer', 'explain', 'reference', 'capture_complete')",
    )?;
    let times: Vec<String> = statement
        .query_map([target.to_string()], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for text in times {
        let moment = timestamps::parse(&text).map_err(|error| {
            StorageError::CorruptRecord(format!("support time {text}: {error}"))
        })?;
        if now - RECENT <= moment && moment <= now {
            return Ok(true);
        }
    }
    Ok(false)
}
