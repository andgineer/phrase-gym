use std::collections::BTreeSet;

use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::error::CoreError;

pub(crate) fn reported_cues(
    connection: &Connection,
    target: Uuid,
    revision: i64,
) -> Result<BTreeSet<String>, CoreError> {
    let mut statement = connection
        .prepare("SELECT cue_id FROM cue_report WHERE target_id = ?1 AND revision = ?2")?;
    let cues = statement
        .query_map(params![target.to_string(), revision], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(cues)
}

pub(crate) fn is_reported(
    connection: &Connection,
    target: Uuid,
    revision: i64,
    cue: &str,
) -> Result<bool, CoreError> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM cue_report \
         WHERE target_id = ?1 AND revision = ?2 AND cue_id = ?3)",
        params![target.to_string(), revision, cue],
        |row| row.get(0),
    )?)
}

// The first report keeps its reason; later reports of the same cue revision change nothing.
pub(crate) fn record(
    connection: &Connection,
    target: Uuid,
    revision: i64,
    cue: &str,
    reason: &str,
) -> Result<(), CoreError> {
    connection.execute(
        "INSERT INTO cue_report (target_id, revision, cue_id, reason) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT DO NOTHING",
        params![target.to_string(), revision, cue, reason],
    )?;
    Ok(())
}
