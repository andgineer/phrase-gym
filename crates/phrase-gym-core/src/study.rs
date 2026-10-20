use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::CoreError;
use crate::schedules::Schedule;

// Next already retains ungraded completion atomically with the attempt; neither lookup exposure
// nor supplied familiarity establishes that this exact content revision was studied.
pub(crate) fn completed(
    connection: &Connection,
    target: Uuid,
    revision: i64,
) -> Result<bool, CoreError> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM practice_attempt WHERE target_id = ?1 AND revision = ?2 \
         AND json_extract(payload, '$.eligibility_reason') = 'ungraded_study' \
         AND json_extract(payload, '$.finalized_at') IS NOT NULL)",
        params![target.to_string(), revision],
        |row| row.get(0),
    )?)
}

// An assessed future schedule is bypassed only with an actual receipt binding that exact schedule
// version to older content. Unknown or same-revision assessed history keeps its due obligation.
pub(crate) fn first_check(
    connection: &Connection,
    target: Uuid,
    revision: i64,
    schedule: Option<&Schedule>,
) -> Result<bool, CoreError> {
    if !completed(connection, target, revision)? {
        return Ok(false);
    }
    let Some(schedule) = schedule.filter(|schedule| schedule.memory_state.is_some()) else {
        return Ok(true);
    };
    let assessed_revision: Option<i64> = connection.query_row(
        "SELECT a.revision FROM review_receipt r JOIN practice_attempt a ON a.id = r.attempt_id \
         WHERE r.target_id = ?1 AND json_extract(r.payload, '$.receipt.schedule_version') = ?2 \
         AND json_extract(r.payload, '$.receipt.schedule_changed') = 1 LIMIT 1",
        params![target.to_string(), schedule.version], |row| row.get(0),
    ).optional()?;
    Ok(assessed_revision.is_some_and(|assessed| assessed != revision))
}
