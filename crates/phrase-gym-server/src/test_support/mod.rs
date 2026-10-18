mod interception;
mod routes;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use phrase_gym_core::storage::Database;
use tokio::sync::watch;
use uuid::Uuid;

pub(crate) use interception::intercept;
pub(crate) use routes::routes;

#[derive(Default)]
pub(super) struct Faults {
    pending: bool,
    lose_rating: bool,
    recording: Option<watch::Sender<bool>>,
    session: Option<(Option<Uuid>, watch::Sender<bool>)>,
    session_held: bool,
}

pub(crate) struct Controls {
    database: Arc<Database>,
    faults: Mutex<Faults>,
}

impl Controls {
    pub(crate) fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            faults: Mutex::default(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Faults> {
        self.faults.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn reset(&self) -> rusqlite::Result<()> {
        let mut faults = self.lock();
        if let Some(gate) = faults.recording.take() {
            let _ = gate.send(true);
        }
        if let Some((_, gate)) = faults.session.take() {
            let _ = gate.send(true);
        }
        *faults = Faults::default();
        drop(faults);
        self.database.write(|transaction| {
            transaction.pragma_update(None, "defer_foreign_keys", "ON")?;
            let tables = transaction.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")?
                .query_map([], |row| row.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
            for table in tables {
                transaction.execute(&format!("DELETE FROM \"{}\"", table.replace('"', "\"\"")), [])?;
            }
            Ok(())
        })
    }
}

async fn wait(mut gate: watch::Receiver<bool>) {
    if !*gate.borrow() {
        let _ = gate.changed().await;
    }
}
