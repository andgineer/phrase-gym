use std::collections::{BTreeSet, HashMap};

use uuid::Uuid;

use crate::error::CoreError;
use crate::storage::Database;

use super::records::{self, MediaRecord};
use super::store::MediaStore;

// Media rows are immutable, so rows read in one snapshot and files checked afterwards stay
// consistent, and no file I/O happens inside a database transaction.
#[derive(Debug, Default)]
pub struct MediaIndex {
    entries: HashMap<Uuid, (MediaRecord, bool)>,
}

impl MediaIndex {
    pub fn load(
        database: &Database,
        store: &MediaStore,
        ids: impl IntoIterator<Item = Uuid>,
    ) -> Result<Self, CoreError> {
        let wanted: BTreeSet<Uuid> = ids.into_iter().collect();
        if wanted.is_empty() {
            return Ok(Self::default());
        }
        let found = database.read(|transaction| {
            wanted
                .iter()
                .filter_map(|id| records::find(transaction, *id).transpose())
                .collect::<Result<Vec<_>, CoreError>>()
        })?;
        let entries = found
            .into_iter()
            .map(|record| {
                let available = record.available_in(store);
                (record.id, (record, available))
            })
            .collect();
        Ok(Self { entries })
    }

    pub fn record(&self, id: Uuid) -> Option<&MediaRecord> {
        self.entries.get(&id).map(|(record, _)| record)
    }

    pub fn available(&self, id: Uuid) -> bool {
        self.entries
            .get(&id)
            .is_some_and(|(_, available)| *available)
    }
}
