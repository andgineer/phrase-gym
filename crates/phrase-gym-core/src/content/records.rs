use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::content::TargetInput;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::CoreError;
use crate::storage::StorageError;

#[derive(Clone, Debug)]
pub struct TargetRow {
    pub id: Uuid,
    pub language: String,
    pub kind: String,
    pub meaning_key: String,
    pub recall_expression: String,
    pub current_revision: i64,
}

pub(crate) fn uuid(text: &str) -> Result<Uuid, CoreError> {
    Uuid::parse_str(text)
        .map_err(|error| StorageError::CorruptRecord(format!("identifier {text}: {error}")).into())
}

pub(crate) fn language(code: &str) -> Result<Language, CoreError> {
    Language::from_code(code)
        .ok_or_else(|| StorageError::CorruptRecord(format!("language {code}")).into())
}

pub(crate) fn parse<T: serde::de::DeserializeOwned>(
    what: &str,
    text: &str,
) -> Result<T, CoreError> {
    serde_json::from_str(text)
        .map_err(|error| StorageError::CorruptRecord(format!("{what}: {error}")).into())
}

const TARGET_COLUMNS: &str = "id, language, kind, meaning_key, recall_expression, current_revision";

fn target_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(String, TargetRow)> {
    Ok((
        row.get(0)?,
        TargetRow {
            id: Uuid::nil(),
            language: row.get(1)?,
            kind: row.get(2)?,
            meaning_key: row.get(3)?,
            recall_expression: row.get(4)?,
            current_revision: row.get(5)?,
        },
    ))
}

fn with_id((id, row): (String, TargetRow)) -> Result<TargetRow, CoreError> {
    Ok(TargetRow {
        id: uuid(&id)?,
        ..row
    })
}

pub fn target(connection: &Connection, id: Uuid) -> Result<Option<TargetRow>, CoreError> {
    connection
        .query_row(
            &format!("SELECT {TARGET_COLUMNS} FROM target WHERE id = ?1"),
            [id.to_string()],
            target_row,
        )
        .optional()?
        .map(with_id)
        .transpose()
}

pub fn sourced_target(
    connection: &Connection,
    namespace: &str,
    source_id: &str,
) -> Result<Option<TargetRow>, CoreError> {
    connection
        .query_row(
            &format!(
                "SELECT {TARGET_COLUMNS} FROM target WHERE id = \
                 (SELECT target_id FROM source_item WHERE namespace = ?1 AND source_id = ?2)"
            ),
            params![namespace, source_id],
            target_row,
        )
        .optional()?
        .map(with_id)
        .transpose()
}

pub fn revision_payload(
    connection: &Connection,
    target: Uuid,
    revision: i64,
) -> Result<Option<String>, CoreError> {
    Ok(connection
        .query_row(
            "SELECT payload FROM target_revision WHERE target_id = ?1 AND revision = ?2",
            params![target.to_string(), revision],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn content(
    connection: &Connection,
    target: Uuid,
    revision: i64,
) -> Result<Option<TargetInput>, CoreError> {
    revision_payload(connection, target, revision)?
        .map(|payload| parse(&format!("target {target} revision {revision}"), &payload))
        .transpose()
}
