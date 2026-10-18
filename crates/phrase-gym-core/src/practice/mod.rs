mod dialogue;
pub(crate) mod events;
mod evidence;
mod observations;
mod repairs;
mod reviews;
mod support;

use std::sync::Arc;

use phrase_gym_contracts::practice::{EventBatch, EventReceipt};
use phrase_gym_contracts::sessions::PracticeBatchReceipt;
use uuid::Uuid;

use crate::clock::Clock;
use crate::content::records;
use crate::error::CoreError;
use crate::scheduler::ReviewPolicy;
use crate::sessions::store;
use crate::storage::Database;

const MAX_BATCH: usize = 100;

pub struct Practice {
    database: Arc<Database>,
    clock: Arc<dyn Clock>,
    policy: ReviewPolicy,
}

fn not_found() -> CoreError {
    CoreError::NotFound("Session not found".to_owned())
}

impl Practice {
    pub fn new(database: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        Self {
            database,
            clock,
            policy: ReviewPolicy::default(),
        }
    }

    // A batch commits as a whole: any conflict or invalid event rolls back every earlier event in it.
    pub fn submit(
        &self,
        session_id: Uuid,
        batch: &EventBatch,
    ) -> Result<PracticeBatchReceipt, CoreError> {
        if batch.events.is_empty() || batch.events.len() > MAX_BATCH {
            return Err(CoreError::Invalid(format!(
                "A batch carries 1-{MAX_BATCH} events"
            )));
        }
        let checked = batch
            .events
            .iter()
            .map(events::check)
            .collect::<Result<Vec<_>, _>>()?;
        self.database.write(|transaction| {
            let mut session = store::find(transaction, session_id)?.ok_or_else(not_found)?;
            let now = self.clock.now();
            let receipts = checked
                .iter()
                .map(|event| events::accept(transaction, &mut session, event, now, &self.policy))
                .collect::<Result<Vec<_>, _>>()?;
            store::update(transaction, &session)?;
            Ok(PracticeBatchReceipt {
                receipts,
                session: store::view(transaction, &session)?,
            })
        })
    }

    pub fn receipts(&self, session_id: Uuid) -> Result<Vec<EventReceipt>, CoreError> {
        self.database.read(|transaction| {
            store::find(transaction, session_id)?.ok_or_else(not_found)?;
            let mut statement = transaction.prepare(
                "SELECT id, receipt FROM practice_event WHERE session_id = ?1 ORDER BY sequence",
            )?;
            let rows: Vec<(String, String)> = statement
                .query_map([session_id.to_string()], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })?
                .collect::<Result<_, _>>()?;
            rows.into_iter()
                .map(|(id, receipt)| records::parse(&format!("receipt of event {id}"), &receipt))
                .collect()
        })
    }
}
