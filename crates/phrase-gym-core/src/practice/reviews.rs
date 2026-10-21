use chrono::{DateTime, TimeDelta, Utc};
use phrase_gym_contracts::practice::{AttemptEvidence, EventReceipt, RecallRating};
use phrase_gym_contracts::sessions::PlannedTarget;
use rusqlite::{Connection, params};
use serde_json::{Value, json};

use crate::content::records;
use crate::error::CoreError;
use crate::prompts::{history, reports};
use crate::scheduler::{POLICY_VERSION, ReviewPolicy};
use crate::schedules::{self, Schedule};
use crate::sessions::store::SessionRow;
use crate::storage::StorageError;
use crate::{study, timestamps};

const LEARNING_CHECK: TimeDelta = TimeDelta::days(1);

pub(crate) struct Review<'a> {
    pub session: &'a SessionRow,
    pub item: &'a PlannedTarget,
    pub now: DateTime<Utc>,
}

fn group_consumed(connection: &Connection, item: &PlannedTarget) -> Result<bool, CoreError> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM review_receipt WHERE group_id = ?1)",
        [item.review_group_id.to_string()],
        |row| row.get(0),
    )?)
}

// The first exclusion that applies explains why a rating keeps its history without grading memory.
fn exclusion(
    connection: &Connection,
    review: &Review<'_>,
    evidence: &AttemptEvidence,
    schedule: Option<&Schedule>,
) -> Result<Option<&'static str>, CoreError> {
    let item = review.item;
    let (target, revision) = (item.phrase.id, item.phrase.revision);
    let current = records::target(connection, target)?.map(|row| row.current_revision);
    let reason = if review.session.manifest.voluntary {
        "voluntary_practice"
    } else if review.session.manifest.policy_version != POLICY_VERSION {
        "stale_policy"
    } else if !review.session.status.is_open() {
        "closed_session"
    } else if current != Some(revision) {
        "stale_content"
    } else if reports::is_reported(connection, target, revision, &item.cue_id)? {
        "reported_cue"
    } else if evidence.technical_failure || evidence.capture_pending {
        "technical_failure"
    } else if (evidence.original.as_ref()).is_none_or(|original| original.supported) {
        "supported"
    } else if evidence.repair {
        "repair"
    } else if group_consumed(connection, item)? {
        "review_group_already_finalized"
    } else if schedule.map_or(0, |due| due.version) != item.schedule_version {
        "stale_schedule"
    } else if schedule.is_some_and(|due| due.due_at > review.now)
        && !(item.phrase.english.is_some()
            && study::first_check(connection, target, revision, schedule)?)
    {
        "early_practice"
    } else {
        return Ok(None);
    };
    Ok(Some(reason))
}

fn memory(state: Option<&str>) -> Result<Value, CoreError> {
    state.map_or(Ok(Value::Null), |text| records::parse("memory state", text))
}

// Rating, schedule change and review receipt commit together in the caller's transaction.
pub(crate) fn finalize(
    connection: &Connection,
    review: &Review<'_>,
    evidence: &mut AttemptEvidence,
    receipt: &mut EventReceipt,
    policy: &ReviewPolicy,
) -> Result<(), CoreError> {
    let rating = evidence
        .rating
        .ok_or_else(|| CoreError::Invalid("Finalization requires a rating".to_owned()))?;
    let (item, now) = (review.item, review.now);
    let mut schedule = schedules::find(connection, item.phrase.id)?;
    let reason = exclusion(connection, review, evidence, schedule.as_ref())?;
    let group_free = !group_consumed(connection, item)?;
    let before = schedule.as_ref().and_then(|due| due.memory_state.clone());
    let mut outcome = reason.unwrap_or("eligible");
    if reason.is_none() {
        let (state, due_at) = policy
            .review(before.as_deref(), rating, now)
            .map_err(|error| StorageError::CorruptRecord(error.to_string()))?;
        let updated = Schedule {
            version: schedule.as_ref().map_or(0, |due| due.version) + 1,
            due_at,
            memory_state: Some(state),
        };
        schedules::save(connection, item.phrase.id, &updated, POLICY_VERSION)?;
        schedule = Some(updated);
        receipt.schedule_changed = true;
    } else if reason == Some("supported") && schedule.is_none() && group_free {
        let check = Schedule {
            version: 1,
            due_at: now + LEARNING_CHECK,
            memory_state: None,
        };
        schedules::save(connection, item.phrase.id, &check, POLICY_VERSION)?;
        schedule = Some(check);
        receipt.schedule_changed = true;
        outcome = "supported_learning_check";
    }
    evidence.finalized_at = Some(timestamps::isoformat(now));
    evidence.eligibility_reason = Some(outcome.to_owned());
    receipt.eligibility_reason = Some(outcome.to_owned());
    if let Some(schedule) = &schedule {
        receipt.schedule_version = Some(schedule.version);
        receipt.due_at = Some(timestamps::isoformat(schedule.due_at));
    }
    if group_free && matches!(reason, None | Some("supported")) {
        let parameters = policy.scheduler().parameters();
        let payload = json!({
            "receipt": receipt,
            "policy_version": POLICY_VERSION,
            "parameters": {
                "weights": parameters.weights,
                "desired_retention": parameters.desired_retention,
                "maximum_interval": parameters.maximum_interval,
            },
            "before": memory(before.as_deref())?,
            "after": memory(schedule.as_ref().and_then(|due| due.memory_state.as_deref()))?,
        });
        connection.execute(
            "INSERT INTO review_receipt (group_id, target_id, attempt_id, payload) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                item.review_group_id.to_string(),
                item.phrase.id.to_string(),
                evidence.id.to_string(),
                payload.to_string(),
            ],
        )?;
    }
    let unaided = evidence
        .original
        .as_ref()
        .is_some_and(|original| !original.supported);
    if unaided && !evidence.technical_failure && matches!(reason, None | Some("repair")) {
        history::record_outcome(
            connection,
            item.phrase.id,
            &item.hidden_chunks,
            rating != RecallRating::NotYet,
        )?;
    }
    Ok(())
}
