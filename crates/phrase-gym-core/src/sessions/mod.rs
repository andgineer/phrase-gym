pub(crate) mod attempts;
pub(crate) mod store;
#[cfg(all(test, feature = "test-support"))]
mod tests;

use std::collections::BTreeSet;
use std::sync::Arc;

use phrase_gym_contracts::sessions::{
    PracticeSession, SessionStart, SessionStatus, SessionTransition,
};
use rusqlite::Connection;
use uuid::Uuid;

use crate::clock::Clock;
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::media::index::MediaIndex;
use crate::media::store::MediaStore;
use crate::planning::{self, queue, scope};
use crate::storage::Database;

use store::SessionRow;

// Content published between the media check and the start transaction is re-checked a few times;
// after that, media nobody checked counts as unavailable.
const MEDIA_ATTEMPTS: usize = 3;

enum Started {
    Session(Box<PracticeSession>),
    NeedsMedia(BTreeSet<Uuid>),
}

pub struct Sessions {
    database: Arc<Database>,
    store: Arc<MediaStore>,
    clock: Arc<dyn Clock>,
}

fn not_found() -> CoreError {
    CoreError::NotFound("Session not found".to_owned())
}

fn session_started(session: PracticeSession) -> Started {
    Started::Session(Box::new(session))
}

fn replay(
    connection: &Connection,
    existing: &SessionRow,
    payload: &str,
) -> Result<PracticeSession, CoreError> {
    if existing.start_payload != payload {
        return Err(CoreError::Conflict(
            "Session ID already has different start parameters".to_owned(),
        ));
    }
    store::view(connection, existing)
}

impl Sessions {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            database,
            store,
            clock,
        }
    }

    pub fn current(&self) -> Result<Option<PracticeSession>, CoreError> {
        self.database.read(|transaction| {
            store::open(transaction)?
                .map(|row| store::view(transaction, &row))
                .transpose()
        })
    }

    pub fn get(&self, id: Uuid) -> Result<PracticeSession, CoreError> {
        self.database.read(|transaction| {
            let row = store::find(transaction, id)?.ok_or_else(not_found)?;
            store::view(transaction, &row)
        })
    }

    // Media files are verified before the write transaction so no file I/O holds the writer.
    pub fn start(&self, start: &SessionStart) -> Result<PracticeSession, CoreError> {
        let payload = canonical_json(start);
        let mut started = self.database.read(|transaction| {
            if let Some(existing) = store::find(transaction, start.id)? {
                return replay(transaction, &existing, &payload).map(session_started);
            }
            // Scope errors are reported by the start transaction in their documented order.
            match scope::resolve(transaction, start) {
                Ok(scope) => Ok(Started::NeedsMedia(scope.media_ids())),
                Err(CoreError::NotFound(_) | CoreError::Invalid(_)) => {
                    Ok(Started::NeedsMedia(BTreeSet::new()))
                }
                Err(error) => Err(error),
            }
        })?;
        for attempt in 1..=MEDIA_ATTEMPTS {
            let media = match started {
                Started::Session(session) => return Ok(*session),
                Started::NeedsMedia(ids) => MediaIndex::load(&self.database, &self.store, ids)?,
            };
            let final_attempt = attempt == MEDIA_ATTEMPTS;
            started = self.database.write(|transaction| {
                self.begin(transaction, start, &payload, &media, final_attempt)
            })?;
        }
        match started {
            Started::Session(session) => Ok(*session),
            Started::NeedsMedia(_) => unreachable!("the final attempt plans with the media it has"),
        }
    }

    fn begin(
        &self,
        transaction: &Connection,
        start: &SessionStart,
        payload: &str,
        media: &MediaIndex,
        final_attempt: bool,
    ) -> Result<Started, CoreError> {
        if let Some(existing) = store::find(transaction, start.id)? {
            return replay(transaction, &existing, payload).map(session_started);
        }
        let current = store::open(transaction)?;
        if start.voluntary && current.is_some() {
            return Err(CoreError::Conflict(
                "Resume the unfinished session before starting Practice now".to_owned(),
            ));
        }
        if start.takeover_session_id != current.as_ref().map(|row| row.id) {
            return Err(CoreError::Conflict(
                "Resume the open session or explicitly take over its current ID".to_owned(),
            ));
        }
        let scope = scope::resolve(transaction, start)?;
        let needed = scope.media_ids();
        if !final_attempt && !media.covers(&needed) {
            return Ok(Started::NeedsMedia(needed));
        }
        let now = self.clock.now();
        let manifest = planning::plan(transaction, start, scope, now, media)?;
        if manifest.targets.is_empty() {
            return Err(CoreError::Conflict(
                "No practice-ready targets in this scope; reference audio and valid cues are required".to_owned(),
            ));
        }
        let queue = queue::initial(&manifest)?;
        if let Some(mut replaced) = current {
            replaced.status = SessionStatus::Superseded;
            replaced.touch(now);
            store::update(transaction, &replaced)?;
        }
        let row = SessionRow {
            id: start.id,
            start_payload: payload.to_owned(),
            manifest,
            status: SessionStatus::Active,
            version: 1,
            created_at: now,
            updated_at: now,
            queue,
            repairs: Vec::new(),
            cursor: 0,
            last_sequence: 0,
        };
        store::insert(transaction, &row)?;
        store::view(transaction, &row).map(session_started)
    }

    pub fn transition(
        &self,
        id: Uuid,
        transition: &SessionTransition,
    ) -> Result<PracticeSession, CoreError> {
        if transition.expected_version < 1 {
            return Err(CoreError::Invalid(
                "expected_version must be at least 1".to_owned(),
            ));
        }
        self.database.write(|transaction| {
            let mut row = store::find(transaction, id)?.ok_or_else(not_found)?;
            if !row.status.is_open() {
                return Err(CoreError::Conflict(
                    "A closed session cannot resume".to_owned(),
                ));
            }
            let status = SessionStatus::from(transition.status);
            let retried = row.status == status && row.version == transition.expected_version + 1;
            if !retried {
                if row.version != transition.expected_version {
                    return Err(CoreError::Conflict(
                        "Session changed; reload its current version".to_owned(),
                    ));
                }
                if row.status != status {
                    row.status = status;
                    row.touch(self.clock.now());
                    store::update(transaction, &row)?;
                }
            }
            store::view(transaction, &row)
        })
    }
}
