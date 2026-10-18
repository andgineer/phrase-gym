use chrono::{DateTime, SubsecRound, TimeDelta, Utc};
use phrase_gym_contracts::practice::{EventReceipt, PracticeInput, PracticeKind};
use rusqlite::{Connection, OptionalExtension, params};

use crate::content::records;
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::scheduler::ReviewPolicy;
use crate::sessions::attempts;
use crate::sessions::store::SessionRow;
use crate::storage::StorageError;
use crate::timestamps;

use super::observations;

const CLOCK_SKEW: TimeDelta = TimeDelta::minutes(5);
const MAX_REASON: usize = 1000;

pub(crate) struct Checked<'a> {
    pub event: &'a PracticeInput,
    pub claimed: DateTime<Utc>,
}

pub(crate) fn check(event: &PracticeInput) -> Result<Checked<'_>, CoreError> {
    let invalid = |detail: &str| Err(CoreError::Invalid(detail.to_owned()));
    if event.sequence < 1 || event.step < 0 {
        return invalid("sequence must be at least 1 and step at least 0");
    }
    if (event.kind == PracticeKind::Rate) != event.rating.is_some() {
        return invalid("Only a rate event supplies a rating");
    }
    if event
        .reason
        .as_ref()
        .is_some_and(|reason| reason.chars().count() > MAX_REASON)
    {
        return invalid("reason must be at most 1000 characters");
    }
    if event.kind == PracticeKind::Report && event.reason.as_deref().is_none_or(str::is_empty) {
        return invalid("Cue reports require a reason");
    }
    let claimed = timestamps::parse(&event.occurred_at)
        .map_err(|_| {
            CoreError::Invalid("occurred_at must be a date-time with a time zone".to_owned())
        })?
        .trunc_subsecs(6);
    Ok(Checked { event, claimed })
}

// Equal instants written differently are the same payload, so a retry compares by meaning.
fn retained_payload(checked: &Checked<'_>) -> String {
    canonical_json(&PracticeInput {
        occurred_at: timestamps::isoformat(checked.claimed),
        ..checked.event.clone()
    })
}

fn previous(
    connection: &Connection,
    event: &PracticeInput,
) -> Result<Option<(String, String, String)>, CoreError> {
    Ok(connection
        .query_row(
            "SELECT session_id, payload, receipt FROM practice_event WHERE id = ?1",
            [event.id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?)
}

// Client time is trusted only within the session's span plus five minutes of skew and never moves
// before the previous receipt; every correction stays visible on the receipt.
fn receipt(
    connection: &Connection,
    session: &SessionRow,
    checked: &Checked<'_>,
    now: DateTime<Utc>,
) -> Result<EventReceipt, CoreError> {
    let start = session.created_at;
    let claimed = checked.claimed;
    let plausible = start - CLOCK_SKEW <= claimed && claimed <= now + CLOCK_SKEW;
    let mut occurred = if plausible {
        start.max(claimed.min(now))
    } else {
        now
    };
    let prior: Option<String> = connection
        .query_row(
            "SELECT occurred_at FROM practice_event WHERE session_id = ?1 AND sequence = ?2",
            params![session.id.to_string(), session.last_sequence],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(prior) = prior {
        let prior = timestamps::parse(&prior)
            .map_err(|error| StorageError::CorruptRecord(format!("event time: {error}")))?;
        occurred = occurred.max(prior);
    }
    let step = usize::try_from(checked.event.step).unwrap_or(usize::MAX);
    Ok(EventReceipt {
        id: checked.event.id,
        sequence: checked.event.sequence,
        attempt_id: attempts::attempt_id(session.id, step),
        received_at: timestamps::isoformat(now),
        occurred_at: timestamps::isoformat(occurred),
        clock_adjusted: claimed != occurred,
        capture_supported: None,
        schedule_changed: false,
        schedule_version: None,
        due_at: None,
        eligibility_reason: None,
    })
}

pub(crate) fn accept(
    connection: &Connection,
    session: &mut SessionRow,
    checked: &Checked<'_>,
    now: DateTime<Utc>,
    policy: &ReviewPolicy,
) -> Result<EventReceipt, CoreError> {
    let event = checked.event;
    let payload = retained_payload(checked);
    if let Some((owner, stored, receipt)) = previous(connection, event)? {
        if stored != payload || owner != session.id.to_string() {
            return Err(CoreError::Conflict(
                "Event ID already has a different payload or session".to_owned(),
            ));
        }
        return records::parse(&format!("receipt of event {}", event.id), &receipt);
    }
    if event.sequence != session.last_sequence + 1 {
        return Err(CoreError::Conflict(format!(
            "Expected event sequence {}",
            session.last_sequence + 1
        )));
    }
    let addressed = usize::try_from(event.step).ok();
    if addressed != Some(session.cursor) || session.cursor >= session.queue.len() {
        return Err(CoreError::Conflict(
            "Event does not address the current session step".to_owned(),
        ));
    }
    let mut receipt = receipt(connection, session, checked, now)?;
    observations::process(connection, session, event, &mut receipt, policy)?;
    session.last_sequence = event.sequence;
    connection.execute(
        "INSERT INTO practice_event \
         (id, session_id, sequence, kind, attempt_id, occurred_at, payload, receipt) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            event.id.to_string(),
            session.id.to_string(),
            event.sequence,
            event.kind.code(),
            receipt.attempt_id.to_string(),
            receipt.occurred_at,
            payload,
            serde_json::to_string(&receipt)
                .unwrap_or_else(|_| unreachable!("receipts always serialize")),
        ],
    )?;
    Ok(receipt)
}

// The kind and bound attempt of an acknowledged event, for checking a take's capture context.
pub(crate) fn binding(
    connection: &Connection,
    id: uuid::Uuid,
) -> Result<Option<(String, uuid::Uuid)>, CoreError> {
    let row: Option<(String, String)> = connection
        .query_row(
            "SELECT kind, attempt_id FROM practice_event WHERE id = ?1",
            [id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    row.map(|(kind, attempt)| Ok((kind, records::uuid(&attempt)?)))
        .transpose()
}
