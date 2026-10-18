use std::collections::HashSet;
use std::sync::Arc;

use phrase_gym_contracts::media::Reconciliation;

use crate::error::CoreError;
use crate::storage::Database;

use super::records;
use super::store::MediaStore;

pub struct MediaReconciler {
    database: Arc<Database>,
    store: Arc<MediaStore>,
}

impl MediaReconciler {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>) -> Self {
        Self { database, store }
    }

    // Age alone cannot establish that a transfer or a recovery asset is unreferenced.
    pub fn reconcile(&self) -> Result<Reconciliation, CoreError> {
        let rows = self
            .database
            .read(|transaction| records::all(transaction))?;
        let referenced: HashSet<&str> = rows.iter().map(|row| row.sha256.as_str()).collect();
        let missing_ids = rows
            .iter()
            .filter(|row| !row.available_in(&self.store))
            .map(|row| row.id)
            .collect();
        let orphan_hashes = self
            .store
            .stored_hashes()?
            .into_iter()
            .filter(|name| !referenced.contains(name.as_str()))
            .collect();
        Ok(Reconciliation {
            missing_ids,
            orphan_hashes,
            removed_staging_files: 0,
        })
    }
}
