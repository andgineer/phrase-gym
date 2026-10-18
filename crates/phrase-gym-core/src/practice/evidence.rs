use chrono::{DateTime, Utc};
use phrase_gym_contracts::practice::PracticeKind::{
    Answer, CaptureComplete, Explain, Failure, Rate, Reference, Respond,
};
use phrase_gym_contracts::practice::{
    AttemptEvidence, EventReceipt, OriginalResponse, PracticeInput, PracticeKind,
};
use phrase_gym_contracts::sessions::{PlannedTarget, SessionStep};
use rusqlite::Connection;

use crate::error::CoreError;
use crate::prompts::history;
use crate::sessions::attempts;
use crate::sessions::store::SessionRow;
use crate::timestamps;

use super::support;

pub(crate) fn create(
    connection: &Connection,
    session: &SessionRow,
    item: &PlannedTarget,
    step: &SessionStep,
    now: DateTime<Utc>,
) -> Result<AttemptEvidence, CoreError> {
    let evidence = AttemptEvidence {
        id: attempts::attempt_id(session.id, session.cursor),
        step: i64::try_from(session.cursor).unwrap_or(i64::MAX),
        target_id: item.phrase.id,
        revision: item.phrase.revision,
        cue_id: item.cue_id.clone(),
        review_group_id: item.review_group_id,
        repair: step.repair,
        supported: support::recent(connection, item.phrase.id, now)?,
        original: None,
        revealed: false,
        expanded: false,
        technical_failure: false,
        capture_pending: false,
        rating: None,
        presented_at: timestamps::isoformat(now),
        finalized_at: None,
        eligibility_reason: None,
    };
    attempts::insert(connection, session.id, &evidence)?;
    history::record_presentation(connection, item.phrase.id, &item.hidden_chunks, now)?;
    Ok(evidence)
}

// Support is re-checked at every event until the original response freezes it.
pub(crate) fn refresh_support(
    connection: &Connection,
    evidence: &mut AttemptEvidence,
    kind: PracticeKind,
    receipt: &mut EventReceipt,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    if evidence.original.is_none() && !evidence.supported {
        evidence.supported = support::recent(connection, evidence.target_id, now)?;
    }
    if kind == PracticeKind::Respond {
        receipt.capture_supported = Some(evidence.supported || evidence.revealed);
    }
    Ok(())
}

// The first response snapshots modality and support before this event's own exposure applies.
pub(crate) fn observe(evidence: &mut AttemptEvidence, event: &PracticeInput, now: DateTime<Utc>) {
    let first_response = evidence.original.is_none();
    if matches!(event.kind, Respond | Answer | Rate) && evidence.original.is_none() {
        evidence.original = Some(OriginalResponse {
            modality: event.modality,
            supported: evidence.supported,
            occurred_at: timestamps::isoformat(now),
        });
    }
    match event.kind {
        Answer | Explain | Reference => {
            evidence.supported = true;
            evidence.revealed = true;
            evidence.expanded |= event.kind == Explain;
        }
        Respond => {
            // Distinct captures abandon an unfinished original; ID retries never reach this step.
            evidence.technical_failure |= evidence.capture_pending;
            evidence.capture_pending = first_response;
        }
        CaptureComplete => {
            evidence.revealed = true;
            evidence.capture_pending = false;
        }
        Failure => {
            evidence.technical_failure |= first_response || evidence.capture_pending;
            evidence.capture_pending = false;
        }
        _ => {}
    }
}
