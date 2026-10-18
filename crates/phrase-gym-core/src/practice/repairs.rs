use std::collections::BTreeSet;

use chrono::{DateTime, TimeDelta, Utc};
use phrase_gym_contracts::practice::{AttemptEvidence, RecallRating};
use phrase_gym_contracts::sessions::{SessionStatus, SessionStep};
use rusqlite::Connection;

use crate::error::CoreError;
use crate::sessions::attempts;
use crate::sessions::store::{PendingRepair, SessionRow};
use crate::storage::StorageError;
use crate::timestamps;

const MAX_REPAIRS: usize = 2;
const INTERVENING_TARGETS: usize = 2;
const SPACING: TimeDelta = TimeDelta::minutes(5);
const REPAIRABLE: [&str; 4] = [
    "eligible",
    "repair",
    "supported",
    "supported_learning_check",
];

fn extra_steps(queue: &[SessionStep], target_index: usize) -> usize {
    queue
        .iter()
        .filter(|step| step.repair && step.target_index == target_index)
        .count()
}

fn needs_repair(evidence: &AttemptEvidence) -> bool {
    let repairable = evidence
        .eligibility_reason
        .as_deref()
        .is_some_and(|reason| REPAIRABLE.contains(&reason));
    let failed = evidence.rating == Some(RecallRating::NotYet);
    let supported = evidence
        .original
        .as_ref()
        .is_some_and(|original| original.supported);
    repairable && (failed || supported)
}

// A repair joins the queue only after five minutes and two other finalized targets; nothing waits
// for it, so a short session completes without its pending repairs.
pub(crate) fn advance(
    connection: &Connection,
    session: &mut SessionRow,
    evidence: &AttemptEvidence,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    if !session.status.is_open() {
        return Ok(());
    }
    let step = session.queue[session.cursor].clone();
    let mut pending = std::mem::take(&mut session.repairs);
    if needs_repair(evidence) && extra_steps(&session.queue, step.target_index) < MAX_REPAIRS {
        pending.push(PendingRepair {
            target_index: step.target_index,
            after_step: session.cursor,
            target_id: evidence.target_id,
            eligible_at: timestamps::isoformat(now + SPACING),
        });
    }
    session.cursor += 1;
    let attempts = attempts::of_session(connection, session.id)?;
    for repair in pending {
        if extra_steps(&session.queue, repair.target_index) >= MAX_REPAIRS {
            continue;
        }
        let others: BTreeSet<_> = attempts
            .iter()
            .filter(|attempt| {
                usize::try_from(attempt.step).is_ok_and(|step| step > repair.after_step)
                    && attempt.finalized_at.is_some()
                    && attempt.target_id != repair.target_id
            })
            .map(|attempt| attempt.target_id)
            .collect();
        let eligible_at = timestamps::parse(&repair.eligible_at).map_err(|error| {
            StorageError::CorruptRecord(format!("repair time {}: {error}", repair.eligible_at))
        })?;
        if now >= eligible_at && others.len() >= INTERVENING_TARGETS {
            session.queue.push(SessionStep {
                repair: true,
                ..SessionStep::new(repair.target_index)
            });
        } else {
            session.repairs.push(repair);
        }
    }
    if session.cursor >= session.queue.len() {
        session.status = SessionStatus::Completed;
    }
    session.touch(now);
    Ok(())
}
