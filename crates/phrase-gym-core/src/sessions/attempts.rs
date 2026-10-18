use phrase_gym_contracts::practice::AttemptEvidence;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;

// Attempts are bound to their session step, so retries and late events address the same evidence.
pub(crate) fn attempt_id(session: Uuid, step: usize) -> Uuid {
    Uuid::new_v5(&session, format!("step:{step}").as_bytes())
}

pub(crate) fn find(
    connection: &Connection,
    id: Uuid,
) -> Result<Option<AttemptEvidence>, CoreError> {
    connection
        .query_row(
            "SELECT payload FROM practice_attempt WHERE id = ?1",
            [id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|payload| records::parse(&format!("attempt {id}"), &payload))
        .transpose()
}

pub(crate) fn insert(
    connection: &Connection,
    session: Uuid,
    evidence: &AttemptEvidence,
) -> Result<(), CoreError> {
    connection.execute(
        "INSERT INTO practice_attempt (id, session_id, target_id, revision, step, payload) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            evidence.id.to_string(),
            session.to_string(),
            evidence.target_id.to_string(),
            evidence.revision,
            evidence.step,
            payload(evidence),
        ],
    )?;
    Ok(())
}

pub(crate) fn save(connection: &Connection, evidence: &AttemptEvidence) -> Result<(), CoreError> {
    connection.execute(
        "UPDATE practice_attempt SET payload = ?2 WHERE id = ?1",
        params![evidence.id.to_string(), payload(evidence)],
    )?;
    Ok(())
}

pub(crate) fn of_session(
    connection: &Connection,
    session: Uuid,
) -> Result<Vec<AttemptEvidence>, CoreError> {
    let mut statement = connection
        .prepare("SELECT id, payload FROM practice_attempt WHERE session_id = ?1 ORDER BY step")?;
    let rows: Vec<(String, String)> = statement
        .query_map([session.to_string()], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(id, payload)| records::parse(&format!("attempt {id}"), &payload))
        .collect()
}

fn payload(evidence: &AttemptEvidence) -> String {
    serde_json::to_string(evidence).unwrap_or_else(|_| unreachable!("contracts always serialize"))
}
