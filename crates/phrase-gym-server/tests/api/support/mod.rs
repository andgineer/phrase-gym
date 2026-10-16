pub mod documents;
pub mod scenario;

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use chrono::{DateTime, Utc};
use phrase_gym_core::clock::ManualClock;
use phrase_gym_core::storage::Database;
use phrase_gym_server::{AppState, router};
use tempfile::TempDir;

pub fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub struct TestServer {
    pub router: Router,
    pub clock: Arc<ManualClock>,
    pub database_path: PathBuf,
    _data_dir: TempDir,
}

impl TestServer {
    pub fn start(now: DateTime<Utc>) -> Self {
        let data_dir = tempfile::tempdir().unwrap();
        let database = Database::open(data_dir.path()).unwrap();
        let database_path = database.path().to_owned();
        let clock = Arc::new(ManualClock::new(now));
        let state = AppState::new(Arc::new(database), clock.clone());
        Self {
            router: router(state),
            clock,
            database_path,
            _data_dir: data_dir,
        }
    }
}
