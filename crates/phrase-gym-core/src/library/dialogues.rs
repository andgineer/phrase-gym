use std::collections::BTreeMap;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::library::Dialogue;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;

#[derive(Deserialize)]
struct Bindings {
    ids: BTreeMap<String, Uuid>,
    revisions: BTreeMap<String, i64>,
}

const SELECT: &str = "SELECT d.id, r.revision, d.language, r.payload, r.bindings FROM dialogue d \
                      JOIN dialogue_revision r ON r.dialogue_id = d.id";

type Row = (String, i64, String, String, String);

fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn dialogue((id, revision, language, payload, bindings): Row) -> Result<Dialogue, CoreError> {
    let bindings: Bindings = records::parse("dialogue bindings", &bindings)?;
    Ok(Dialogue {
        id: records::uuid(&id)?,
        revision,
        language: records::language(&language)?,
        content: records::parse("dialogue payload", &payload)?,
        target_ids: bindings.ids,
        target_revisions: bindings.revisions,
    })
}

pub(crate) fn page(
    connection: &Connection,
    language: Language,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Dialogue>, i64), CoreError> {
    let total: i64 = connection.query_row(
        "SELECT count(*) FROM dialogue WHERE language = ?1",
        [language.code()],
        |row| row.get(0),
    )?;
    let mut statement = connection.prepare(&format!(
        "{SELECT} AND r.revision = d.current_revision WHERE d.language = ?1 \
         ORDER BY d.source_id, d.id LIMIT ?2 OFFSET ?3"
    ))?;
    let rows: Vec<Row> = statement
        .query_map(params![language.code(), limit, offset], row)?
        .collect::<Result<_, _>>()?;
    let items = rows.into_iter().map(dialogue).collect::<Result<_, _>>()?;
    Ok((items, total))
}

pub(crate) fn one(
    connection: &Connection,
    id: Uuid,
    language: Language,
    revision: Option<i64>,
) -> Result<Dialogue, CoreError> {
    let found = connection
        .query_row(
            &format!(
                "{SELECT} AND r.revision = coalesce(?3, d.current_revision) \
                 WHERE d.id = ?1 AND d.language = ?2"
            ),
            params![id.to_string(), language.code(), revision],
            row,
        )
        .optional()?;
    found.map(dialogue).transpose()?.ok_or_else(|| {
        CoreError::NotFound("Dialogue not found in this learning language".to_owned())
    })
}
