pub mod delivery;
mod input;
pub mod jobs;
mod records;
pub(crate) mod recovery;

use std::sync::Arc;

use phrase_gym_contracts::inbox::{LessonReceipt, LessonStatus, LessonSubmission};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::clock::Clock;
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::media::store::MediaStore;
use crate::storage::Database;

pub const GUIDE_VERSION: &str = "english-claude-demo-v1";

pub struct Inbox {
    database: Arc<Database>,
    store: Arc<MediaStore>,
    clock: Arc<dyn Clock>,
}

impl Inbox {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            database,
            store,
            clock,
        }
    }

    pub fn submit(
        &self,
        actor: &str,
        input: &LessonSubmission,
    ) -> Result<LessonReceipt, CoreError> {
        let request = canonical_json(input);
        if let Some(receipt) = self
            .database
            .read(|tx| records::retry(tx, actor, input.submission_id, &request))?
        {
            return Ok(receipt);
        }
        input::validate(&self.database, &self.store, input)?;
        self.database.write(|tx| {
            if let Some(receipt) = records::retry(tx, actor, input.submission_id, &request)? {
                return Ok(receipt);
            }
            let previous: Option<i64> = tx.query_row(
                "SELECT MAX(generation) FROM inbox_generation WHERE actor = ?1 AND item_id = ?2",
                params![actor, input.item_id], |row| row.get(0),
            )?;
            if previous != input.expected_generation {
                return Err(CoreError::Conflict("Input generation changed; read its receipt before replacing it".into()));
            }
            let profile_revision: i64 = tx.query_row("SELECT MAX(revision) FROM profile_revision", [], |row| row.get(0))?;
            let receipt = LessonReceipt {
                generation_id: Uuid::new_v4(), submission_id: input.submission_id,
                generation: previous.unwrap_or(0) + 1,
                namespace: format!("inbox:{}", Uuid::new_v5(&Uuid::NAMESPACE_URL, canonical_json(&(actor, &input.item_id)).as_bytes())),
                profile_revision, guide_version: GUIDE_VERSION.into(),
                received_at: self.clock.now().to_rfc3339(),
            };
            input::expected_revisions(tx, input, &receipt.namespace)?;
            tx.execute("UPDATE inbox_generation SET status = 'superseded', token = NULL WHERE actor = ?1 AND item_id = ?2 AND status != 'published'", params![actor, input.item_id])?;
            tx.execute("INSERT INTO inbox_generation (id, actor, item_id, generation, submission_id, request, receipt, profile_revision, guide_version, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'waiting')", params![receipt.generation_id.to_string(), actor, input.item_id, receipt.generation, input.submission_id.to_string(), request, canonical_json(&receipt), profile_revision, GUIDE_VERSION])?;
            for phrase in &input.phrases {
                if let Some(id) = phrase.reference_audio_id {
                    tx.execute("INSERT INTO inbox_attachment VALUES (?1, ?2, ?3)", params![receipt.generation_id.to_string(), phrase.source_id, id.to_string()])?;
                }
            }
            Ok(receipt)
        })
    }

    pub fn receipt(
        &self,
        actor: &str,
        submission: Uuid,
    ) -> Result<Option<LessonReceipt>, CoreError> {
        self.database.read(|tx| {
            let receipt: Option<String> = tx
                .query_row(
                    "SELECT receipt FROM inbox_generation WHERE actor = ?1 AND submission_id = ?2",
                    params![actor, submission.to_string()],
                    |row| row.get(0),
                )
                .optional()?;
            receipt
                .map(|receipt| crate::content::records::parse("input receipt", &receipt))
                .transpose()
        })
    }

    pub fn status(&self, actor: &str, id: Uuid) -> Result<LessonStatus, CoreError> {
        self.database.read(|tx| records::status(tx, actor, id))
    }

    pub fn retry(&self, actor: &str, id: Uuid) -> Result<LessonStatus, CoreError> {
        self.database.write(|tx| {
            records::status(tx, actor, id)?;
            let changed = tx.execute("UPDATE inbox_generation SET status = 'waiting', reason = NULL WHERE id = ?1 AND status = 'failed' AND attempts < ?2", params![id.to_string(), jobs::MAX_ATTEMPTS])?;
            if changed == 0 { return Err(CoreError::Conflict("Only failed jobs below the attempt limit may retry".into())); }
            records::status(tx, actor, id)
        })
    }

    pub fn cancel(&self, actor: &str, id: Uuid) -> Result<LessonStatus, CoreError> {
        self.database.write(|tx| {
            let status = records::status(tx, actor, id)?;
            if status.publication.is_some() {
                return Err(CoreError::Conflict("Published lessons cannot be cancelled".into()));
            }
            tx.execute("UPDATE inbox_generation SET status = 'cancelled', token = NULL WHERE id = ?1 AND status != 'superseded'", [id.to_string()])?;
            records::status(tx, actor, id)
        })
    }
}
