use std::sync::Arc;

use chrono::{DateTime, Utc};
use phrase_gym_contracts::exposures::{ExposureInput, ExposureReceipt};
use rusqlite::{OptionalExtension, Transaction, params};

use crate::clock::Clock;
use crate::error::CoreError;
use crate::storage::{Database, StorageError};
use crate::timestamps;

pub struct ExposureService {
    database: Arc<Database>,
    clock: Arc<dyn Clock>,
}

impl ExposureService {
    pub fn new(database: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        Self { database, clock }
    }

    pub fn expose(&self, input: &ExposureInput) -> Result<ExposureReceipt, CoreError> {
        if input.revision < 1 {
            return Err(CoreError::Invalid(
                "Exposure revision must be at least 1".to_owned(),
            ));
        }
        self.database
            .write(|transaction| record(transaction, input, self.clock.now()))
    }
}

fn record(
    transaction: &Transaction<'_>,
    input: &ExposureInput,
    now: DateTime<Utc>,
) -> Result<ExposureReceipt, CoreError> {
    let id = input.id.to_string();
    let previous: Option<(String, String)> = transaction
        .query_row(
            "SELECT payload, received_at FROM exposure WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((payload, received_at)) = previous {
        let stored: ExposureInput = serde_json::from_str(&payload)
            .map_err(|error| StorageError::CorruptRecord(format!("exposure {id}: {error}")))?;
        if stored != *input {
            return Err(CoreError::Conflict(
                "Exposure ID already used with a different payload".to_owned(),
            ));
        }
        return Ok(ExposureReceipt {
            id: input.id,
            received_at,
        });
    }
    let target_id = input.target_id.to_string();
    let known: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM target JOIN target_revision \
         ON target_revision.target_id = target.id \
         WHERE target.id = ?1 AND target.language = ?2 AND target_revision.revision = ?3)",
        params![target_id, input.language.code(), input.revision],
        |row| row.get(0),
    )?;
    if !known {
        return Err(CoreError::Invalid(
            "Exposure references an unknown phrase/language/revision".to_owned(),
        ));
    }
    let payload = serde_json::to_string(input)
        .unwrap_or_else(|_| unreachable!("exposure contracts always serialize"));
    let received_at = timestamps::isoformat(now);
    transaction.execute(
        "INSERT INTO exposure (id, target_id, revision, payload, received_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, target_id, input.revision, payload, received_at],
    )?;
    Ok(ExposureReceipt {
        id: input.id,
        received_at,
    })
}
