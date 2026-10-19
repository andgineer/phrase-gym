use phrase_gym_contracts::inbox::{DeliveryStatus, LessonReceipt, LessonStatus};
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::content::records::parse;
use crate::error::CoreError;

pub(super) fn retry(
    connection: &Connection,
    actor: &str,
    id: Uuid,
    request: &str,
) -> Result<Option<LessonReceipt>, CoreError> {
    let stored: Option<(String, String)> = connection
        .query_row(
            "SELECT request, receipt FROM inbox_generation WHERE actor = ?1 AND submission_id = ?2",
            params![actor, id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    stored
        .map(|(original, receipt)| {
            if original != request {
                return Err(CoreError::Conflict(
                    "Submission identity has different input".into(),
                ));
            }
            parse("input receipt", &receipt)
        })
        .transpose()
}

pub(super) fn status(
    connection: &Connection,
    actor: &str,
    id: Uuid,
) -> Result<LessonStatus, CoreError> {
    let stored: Option<(String, String, Option<String>, Option<String>)> = connection.query_row(
        "SELECT receipt, status, reason, publication FROM inbox_generation WHERE actor = ?1 AND id = ?2",
        params![actor, id.to_string()], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).optional()?;
    let (receipt, status, reason, publication) =
        stored.ok_or_else(|| CoreError::NotFound("Inbox generation not found".into()))?;
    let status: DeliveryStatus = parse("inbox status", &format!("\"{status}\""))?;
    Ok(LessonStatus {
        receipt: parse("input receipt", &receipt)?,
        status,
        reason,
        publication: publication
            .map(|p| parse("publication receipt", &p))
            .transpose()?,
    })
}
