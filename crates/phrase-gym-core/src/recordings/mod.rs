mod validation;

use std::sync::Arc;

use phrase_gym_contracts::recordings::{RecordingInput, RecordingReceipt};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use uuid::Uuid;

use crate::clock::Clock;
use crate::content::records;
use crate::error::CoreError;
use crate::practice::events;
use crate::sessions::attempts;
use crate::storage::{Database, StorageError};
use crate::timestamps;

pub struct Recordings {
    database: Arc<Database>,
    clock: Arc<dyn Clock>,
}

pub struct Delivery {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub sha256: String,
}

#[derive(Deserialize)]
struct Stored {
    captured_at: String,
    mime_type: String,
    complete: bool,
}

struct Row {
    id: String,
    metadata: String,
    sha256: String,
    byte_length: i64,
    received_at: String,
}

impl Row {
    fn stored(&self) -> Result<Stored, CoreError> {
        records::parse(&format!("take {}", self.id), &self.metadata)
    }

    fn receipt(&self) -> Result<RecordingReceipt, CoreError> {
        let stored = self.stored()?;
        Ok(RecordingReceipt {
            id: records::uuid(&self.id)?,
            sha256: self.sha256.clone(),
            byte_length: self.byte_length,
            captured_at: stored.captured_at,
            received_at: self.received_at.clone(),
            complete: stored.complete,
        })
    }
}

const ROW: &str = "SELECT id, metadata, sha256, length(audio), received_at FROM learner_recording";

fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok(Row {
        id: row.get(0)?,
        metadata: row.get(1)?,
        sha256: row.get(2)?,
        byte_length: row.get(3)?,
        received_at: row.get(4)?,
    })
}

fn find(connection: &Connection, id: Uuid) -> Result<Option<Row>, CoreError> {
    Ok(connection
        .query_row(&format!("{ROW} WHERE id = ?1"), [id.to_string()], row)
        .optional()?)
}

impl Recordings {
    pub fn new(database: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        Self { database, clock }
    }

    // Bytes are decoded and hashed before the short write that stores every distinct take.
    pub fn save(&self, input: &RecordingInput) -> Result<RecordingReceipt, CoreError> {
        let take = validation::take(input)?;
        self.database.write(|transaction| {
            if let Some(previous) = find(transaction, input.id)? {
                if previous.metadata != take.metadata || previous.sha256 != take.sha256 {
                    return Err(CoreError::Conflict(
                        "Take ID already has different audio or metadata".to_owned(),
                    ));
                }
                return previous.receipt();
            }
            validation::context(transaction, input)?;
            if let Some(attempt_id) = input.attempt_id.filter(|_| !input.complete)
                && events::original_capture(transaction, attempt_id, input.capture_event_id)?
                && let Some(mut attempt) = attempts::find(transaction, attempt_id)?
            {
                attempt.technical_failure = true;
                attempts::save(transaction, &attempt)?;
            }
            transaction.execute(
                "INSERT INTO learner_recording \
                 (id, target_id, revision, attempt_id, exposure_id, metadata, audio, sha256, received_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    input.id.to_string(),
                    input.target_id.to_string(),
                    input.revision,
                    input.attempt_id.map(|id| id.to_string()),
                    input.exposure_id.map(|id| id.to_string()),
                    take.metadata,
                    take.audio,
                    take.sha256,
                    timestamps::isoformat(self.clock.now()),
                ],
            )?;
            find(transaction, input.id)?
                .ok_or_else(|| CoreError::Conflict("Take was not recorded".to_owned()))?
                .receipt()
        })
    }

    pub fn list(&self, target: Uuid) -> Result<Vec<RecordingReceipt>, CoreError> {
        let mut receipts = self.database.read(|transaction| {
            let mut statement = transaction.prepare(&format!("{ROW} WHERE target_id = ?1"))?;
            let rows: Vec<Row> = statement
                .query_map([target.to_string()], row)?
                .collect::<Result<_, _>>()?;
            rows.iter()
                .map(Row::receipt)
                .collect::<Result<Vec<_>, CoreError>>()
        })?;
        let mut keyed = Vec::with_capacity(receipts.len());
        for receipt in receipts.drain(..) {
            let instant = |text: &str| {
                timestamps::parse(text).map_err(|error| {
                    StorageError::CorruptRecord(format!("take {} time: {error}", receipt.id))
                })
            };
            let key = (
                instant(&receipt.captured_at)?,
                instant(&receipt.received_at)?,
            );
            keyed.push((key, receipt.id.to_string(), receipt));
        }
        keyed.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
        Ok(keyed.into_iter().map(|(_, _, receipt)| receipt).collect())
    }

    pub fn delivery(&self, id: Uuid) -> Result<Delivery, CoreError> {
        self.database.read(|transaction| {
            let row = find(transaction, id)?
                .ok_or_else(|| CoreError::NotFound("Take not found".to_owned()))?;
            let bytes: Vec<u8> = transaction.query_row(
                "SELECT audio FROM learner_recording WHERE id = ?1",
                [id.to_string()],
                |found| found.get(0),
            )?;
            Ok(Delivery {
                bytes,
                mime_type: row.stored()?.mime_type,
                sha256: row.sha256,
            })
        })
    }
}
