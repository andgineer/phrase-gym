use phrase_gym_contracts::practice::PracticeInput;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::error::CoreError;
use crate::sessions::store::SessionRow;

fn introduction(session: &SessionRow, detail: &str) -> Result<(), CoreError> {
    if session.manifest.dialogue.is_none() || session.cursor != 0 {
        return Err(CoreError::Invalid(detail.to_owned()));
    }
    Ok(())
}

// Showing the script exposes every target of the dialogue at the event's bounded time.
pub(crate) fn expose(
    connection: &Connection,
    session: &SessionRow,
    event: &PracticeInput,
    occurred_at: &str,
) -> Result<(), CoreError> {
    introduction(session, "Only a dialogue introduction exposes its script")?;
    for target in &session.manifest.targets {
        connection.execute(
            "INSERT INTO dialogue_exposure (id, session_id, target_id, revision, occurred_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Uuid::new_v5(&event.id, target.phrase.id.to_string().as_bytes()).to_string(),
                session.id.to_string(),
                target.phrase.id.to_string(),
                target.phrase.revision,
                occurred_at,
            ],
        )?;
    }
    Ok(())
}

pub(crate) fn begin(session: &mut SessionRow) -> Result<(), CoreError> {
    introduction(session, "Only a prepared dialogue can begin rehearsal")?;
    session.queue[0].introduced = true;
    Ok(())
}
