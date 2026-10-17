use std::sync::Arc;

use phrase_gym_core::clock::Clock;
#[cfg(feature = "test-support")]
use phrase_gym_core::content::staging::PackageStaging;
use phrase_gym_core::exposures::ExposureService;
use phrase_gym_core::library::Library;
use phrase_gym_core::media::catalog::MediaCatalog;
use phrase_gym_core::media::ingest::MediaIngest;
use phrase_gym_core::media::reconcile::MediaReconciler;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::readiness::Readiness;
use phrase_gym_core::storage::Database;
use tokio::sync::Semaphore;

use crate::error::ApiError;

const MAX_BLOCKING_STORAGE_TASKS: usize = 8;

#[derive(Clone)]
pub struct AppState {
    pub(crate) exposures: Arc<ExposureService>,
    pub(crate) library: Arc<Library>,
    pub(crate) readiness: Arc<Readiness>,
    pub(crate) media_ingest: Arc<MediaIngest>,
    pub(crate) media_catalog: Arc<MediaCatalog>,
    pub(crate) media_reconciler: Arc<MediaReconciler>,
    #[cfg(feature = "test-support")]
    pub(crate) staging: Arc<PackageStaging>,
    storage_slots: Arc<Semaphore>,
}

impl AppState {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            exposures: Arc::new(ExposureService::new(database.clone(), clock.clone())),
            library: Arc::new(Library::new(database.clone(), store.clone())),
            readiness: Arc::new(Readiness::new(
                database.clone(),
                store.clone(),
                clock.clone(),
            )),
            media_ingest: Arc::new(MediaIngest::new(database.clone(), store.clone())),
            media_catalog: Arc::new(MediaCatalog::new(database.clone(), store.clone())),
            #[cfg(feature = "test-support")]
            staging: Arc::new(PackageStaging::new(database.clone(), store.clone(), clock)),
            media_reconciler: Arc::new(MediaReconciler::new(database, store)),
            storage_slots: Arc::new(Semaphore::new(MAX_BLOCKING_STORAGE_TASKS)),
        }
    }

    // Storage and file work runs on the blocking pool, bounded, so request threads never wait on
    // SQLite locks or disk.
    pub(crate) async fn blocking<T: Send + 'static>(
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
