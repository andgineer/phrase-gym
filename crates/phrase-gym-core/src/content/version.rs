use rusqlite::{Connection, params};
use serde_json::json;

use crate::error::CoreError;
use crate::hashing::fingerprint;

fn rows<T>(
    connection: &Connection,
    sql: &str,
    parameters: impl rusqlite::Params,
    map: impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>, CoreError> {
    let mut statement = connection.prepare(sql)?;
    let values = statement
        .query_map(parameters, map)?
        .collect::<Result<_, _>>()?;
    Ok(values)
}

// Staging pins this fingerprint of a namespace's published state; any later change to its
// targets, lessons or dialogues makes the staged package stale.
pub fn content_version(connection: &Connection, namespace: &str) -> Result<String, CoreError> {
    let targets = rows(
        connection,
        "SELECT s.source_id, s.target_id, t.current_revision FROM source_item s \
         JOIN target t ON t.id = s.target_id WHERE s.namespace = ?1 ORDER BY s.source_id",
        [namespace],
        |row| {
            Ok(json!([
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?
            ]))
        },
    )?;
    let lessons = rows(
        connection,
        "SELECT id, title FROM lesson WHERE namespace = ?1 ORDER BY source_id",
        [namespace],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )?
    .into_iter()
    .map(|(id, title)| {
        let members = rows(
            connection,
            "SELECT target_id, position FROM lesson_membership WHERE lesson_id = ?1 \
             ORDER BY position",
            params![id],
            |row| Ok(json!([row.get::<_, String>(0)?, row.get::<_, i64>(1)?])),
        )?;
        Ok(json!([id, title, members]))
    })
    .collect::<Result<Vec<_>, CoreError>>()?;
    let dialogues = rows(
        connection,
        "SELECT id, current_revision FROM dialogue WHERE namespace = ?1 ORDER BY source_id",
        [namespace],
        |row| Ok(json!([row.get::<_, String>(0)?, row.get::<_, i64>(1)?])),
    )?;
    Ok(fingerprint(
        &json!({"targets": targets, "lessons": lessons, "dialogues": dialogues}),
    ))
}
