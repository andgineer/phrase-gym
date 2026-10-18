use chrono::{DateTime, Utc};
use phrase_gym_contracts::practice::{AttemptEvidence, EventReceipt, PracticeInput, PracticeKind};
use phrase_gym_contracts::sessions::PlannedTarget;
use rusqlite::Connection;

use crate::error::CoreError;
use crate::prompts::reports;
use crate::scheduler::ReviewPolicy;
use crate::sessions::attempts;
use crate::sessions::store::SessionRow;
use crate::storage::StorageError;
use crate::timestamps;

use super::reviews::{self, Review};
use super::{dialogue, evidence, repairs};

fn corrupt(session: &SessionRow) -> CoreError {
    StorageError::CorruptRecord(format!("session {} queue", session.id)).into()
}

// Applies one ordered event to the attempt bound to its step; the attempt, schedule and receipt
// changes land in the caller's transaction.
pub(crate) fn process(
    connection: &Connection,
    session: &mut SessionRow,
    event: &PracticeInput,
    receipt: &mut EventReceipt,
    policy: &ReviewPolicy,
) -> Result<(), CoreError> {
    let now = timestamps::parse(&receipt.occurred_at)
        .map_err(|error| StorageError::CorruptRecord(error.to_string()))?;
    match event.kind {
        PracticeKind::DialogueExposure => {
            return dialogue::expose(connection, session, event, &receipt.occurred_at);
        }
        PracticeKind::DialogueBegin => return dialogue::begin(session),
        _ => {}
    }
    let step = session
        .queue
        .get(session.cursor)
        .cloned()
        .ok_or_else(|| corrupt(session))?;
    let item = session
        .manifest
        .targets
        .get(step.target_index)
        .cloned()
        .ok_or_else(|| corrupt(session))?;
    let mut attempt = match attempts::find(connection, receipt.attempt_id)? {
        Some(attempt) => attempt,
        None if event.kind == PracticeKind::Present => {
            evidence::create(connection, session, &item, &step, now)?
        }
        None => {
            return Err(CoreError::Conflict(
                "Present the cue before reporting a response".to_owned(),
            ));
        }
    };
    evidence::refresh_support(connection, &mut attempt, event.kind, receipt, now)?;
    evidence::observe(&mut attempt, event, now);
    match event.kind {
        PracticeKind::Rate => {
            if attempt.rating.is_some() {
                return Err(CoreError::Conflict("Attempt already finalized".to_owned()));
            }
            attempt.rating = event.rating;
            let review = Review {
                session,
                item: &item,
                now,
            };
            reviews::finalize(connection, &review, &mut attempt, receipt, policy)?;
        }
        PracticeKind::Report => report(connection, &item, &mut attempt, event, now)?,
        PracticeKind::Detour => session.queue[session.cursor].dialogue_turn = None,
        _ => {}
    }
    attempts::save(connection, &attempt)?;
    if matches!(event.kind, PracticeKind::Rate | PracticeKind::Report) {
        repairs::advance(connection, session, &attempt, now)?;
    }
    Ok(())
}

// A reported cue leaves the attempt ungraded and excludes that cue revision from later planning.
fn report(
    connection: &Connection,
    item: &PlannedTarget,
    attempt: &mut AttemptEvidence,
    event: &PracticeInput,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    reports::record(
        connection,
        item.phrase.id,
        item.phrase.revision,
        &item.cue_id,
        event.reason.as_deref().unwrap_or_default(),
    )?;
    attempt.eligibility_reason = Some("reported_cue".to_owned());
    attempt.finalized_at = Some(timestamps::isoformat(now));
    Ok(())
}
