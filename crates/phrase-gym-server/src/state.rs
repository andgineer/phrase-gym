use std::sync::Arc;

use phrase_gym_core::clock::Clock;
use phrase_gym_core::exposures::ExposureService;
use phrase_gym_core::storage::Database;
use tokio::sync::Semaphore;

use crate::error::ApiError;

const MAX_BLOCKING_STORAGE_TASKS: usize = 8;

#[derive(Clone)]
pub struct AppState {
    pub(crate) exposures: Arc<ExposureService>,
    storage_slots: Arc<Semaphore>,
}

impl AppState {
    pub fn new(database: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        Self {
            exposures: Arc::new(ExposureService::new(database, clock)),
            storage_slots: Arc::new(Semaphore::new(MAX_BLOCKING_STORAGE_TASKS)),
        }
    }

    // SQLite work runs on the blocking pool so request threads never wait on locks or disk.
    pub(crate) async fn storage<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Result<T, ApiError> {
        let permit = Arc::clone(&self.storage_slots)
            .acquire_owned()
            .await
            .map_err(|_| ApiError::unavailable())?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            work()
        })
        .await
        .map_err(|_| ApiError::internal())
    }
}
