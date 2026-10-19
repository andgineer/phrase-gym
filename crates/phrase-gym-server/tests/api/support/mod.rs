pub mod capture;
pub mod content;
pub mod documents;
pub mod http;
pub mod practice;
pub mod scenario;
pub mod setup;

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use chrono::{DateTime, Utc};
use phrase_gym_core::clock::ManualClock;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::storage::Database;
use phrase_gym_server::{AppState, Authority, Capability, Credential, ServerSettings, router};
use tempfile::TempDir;

pub const TOKEN: &str = "synthetic-test-credential-000000000000000000";

pub fn settings() -> ServerSettings {
    ServerSettings {
        authority: Arc::new(
            Authority::new(vec![Credential {
                actor: "synthetic-tests".into(),
                token: TOKEN.into(),
                capabilities: vec![
                    Capability::Learner,
                    Capability::Worker,
                    Capability::MaterialSubmit,
                    Capability::ProfileRead,
                    Capability::ProfileUpdate,
                ],
            }])
            .unwrap(),
        ),
        ..ServerSettings::loopback().with_hosts([HOST.to_owned()])
    }
}

pub const HOST: &str = "testserver";

pub fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub struct TestServer {
    pub router: Router,
    pub clock: Arc<ManualClock>,
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    _temporary: TempDir,
}

impl TestServer {
    pub fn start(now: DateTime<Utc>) -> Self {
        Self::with_settings(now, &settings())
    }

    pub fn with_settings(now: DateTime<Utc>, settings: &ServerSettings) -> Self {
        let data_dir = tempfile::tempdir().unwrap();
        let database = Database::open(data_dir.path()).unwrap();
        let database_path = database.path().to_owned();
        let clock = Arc::new(ManualClock::new(now));
        let store = MediaStore::open(data_dir.path()).unwrap();
        let database = Arc::new(database);
        phrase_gym_core::profiles::Profiles::new(database.clone(), clock.clone())
            .initialize_current_learner()
            .unwrap();
        let state = AppState::new(database, Arc::new(store), clock.clone());
        Self {
            router: router(
                state,
                &ServerSettings {
                    authority: self::settings().authority,
                    ..settings.clone()
                },
            ),
            clock,
            data_dir: data_dir.path().to_owned(),
            database_path,
            _temporary: data_dir,
        }
    }

    // A fresh application over the same data directory, as after a process restart.
    pub fn restarted(&self) -> Router {
        let database = Database::open(&self.data_dir).unwrap();
        let store = MediaStore::open(&self.data_dir).unwrap();
        let state = AppState::new(Arc::new(database), Arc::new(store), self.clock.clone());
        router(state, &settings())
    }
}
