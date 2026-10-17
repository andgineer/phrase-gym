use std::sync::Arc;

use base64::Engine;
use base64::alphabet::STANDARD;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use phrase_gym_contracts::media::{MediaAsset, MediaUpload};
use rusqlite::params;

use crate::error::CoreError;
use crate::hashing::{canonical_json, sha256_hex};
use crate::storage::Database;

use super::store::MediaStore;
use super::{containers, identity, provenance, records};

const MAX_BASE64_CHARACTERS: usize = 17_000_000;
const DECODER: GeneralPurpose = GeneralPurpose::new(
    &STANDARD,
    GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
);

pub struct MediaIngest {
    database: Arc<Database>,
    store: Arc<MediaStore>,
}

impl MediaIngest {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>) -> Self {
        Self { database, store }
    }

    // Bytes are verified, hashed and moved into place before the short transaction that records
    // them, so the database never refers to a file that was not complete when committed.
    pub fn upload(&self, upload: &MediaUpload) -> Result<MediaAsset, CoreError> {
        provenance::validate(&upload.metadata)?;
        let encoded = &upload.audio_base64;
        if encoded.is_empty() || encoded.len() > MAX_BASE64_CHARACTERS {
            return Err(CoreError::Invalid(format!(
                "audio_base64 must be 1-{MAX_BASE64_CHARACTERS} characters"
            )));
        }
        let audio = DECODER
            .decode(encoded)
            .map_err(|_| CoreError::Invalid("Audio must be valid base64".to_owned()))?;
        containers::validate(&audio, upload.mime_type)?;
        let sha256 = sha256_hex(&audio);
        let id = identity::media_id(&sha256, upload.mime_type, &upload.metadata);
        let staged = self.store.stage(&audio)?;
        self.store.promote(&staged, &sha256)?;
        let record = self.database.write(|transaction| {
            transaction.execute(
                "INSERT INTO reference_media (id, sha256, byte_length, mime_type, metadata) \
                 VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT (id) DO NOTHING",
                params![
                    id.to_string(),
                    sha256,
                    i64::try_from(audio.len()).unwrap_or(i64::MAX),
                    upload.mime_type.code(),
                    canonical_json(&upload.metadata),
                ],
            )?;
            records::find(transaction, id)?
                .ok_or_else(|| CoreError::Conflict("Reference media was not recorded".to_owned()))
        })?;
        let available = record.available_in(&self.store);
        Ok(record.asset(available))
    }
}
