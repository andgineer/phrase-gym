use std::sync::Arc;

use phrase_gym_contracts::media::MediaAsset;
use uuid::Uuid;

use crate::error::CoreError;
use crate::hashing::sha256_hex;
use crate::storage::Database;

use super::records::{self, MediaRecord};
use super::store::MediaStore;

pub struct MediaCatalog {
    database: Arc<Database>,
    store: Arc<MediaStore>,
}

pub struct Delivery {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub sha256: String,
}

impl MediaCatalog {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>) -> Self {
        Self { database, store }
    }

    fn record(&self, id: Uuid) -> Result<MediaRecord, CoreError> {
        self.database
            .read(|transaction| records::find(transaction, id))?
            .ok_or_else(|| CoreError::NotFound("Reference media not found".to_owned()))
    }

    pub fn asset(&self, id: Uuid) -> Result<MediaAsset, CoreError> {
        let record = self.record(id)?;
        let available = record.available_in(&self.store);
        Ok(record.asset(available))
    }

    // Only bytes that still match the recorded hash are served; damage is reported, never hidden.
    pub fn delivery(&self, id: Uuid) -> Result<Delivery, CoreError> {
        let record = self.record(id)?;
        let bytes = self
            .store
            .read(&record.sha256)
            .ok()
            .filter(|bytes| {
                i64::try_from(bytes.len()) == Ok(record.byte_length)
                    && sha256_hex(bytes) == record.sha256
            })
            .ok_or_else(|| {
                CoreError::Unavailable(
                    "Reference audio is missing or damaged; re-upload the original clip".to_owned(),
                )
            })?;
        Ok(Delivery {
            bytes,
            mime_type: record.mime_type,
            sha256: record.sha256,
        })
    }
}
