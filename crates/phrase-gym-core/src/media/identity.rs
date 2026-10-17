use phrase_gym_contracts::media::{MediaMetadata, ReferenceMime};
use uuid::Uuid;

use crate::hashing::canonical_json;

const MEDIA_NAMESPACE: Uuid = Uuid::from_u128(0x1a6d_9d67_c9b9_4d73_9e38_6dd0_5a3c_1d66);

// Identity covers the exact bytes, their declared container and the complete canonical provenance,
// so re-preparing the same text with another voice or setting yields a distinct asset.
pub fn media_id(sha256: &str, mime: ReferenceMime, metadata: &MediaMetadata) -> Uuid {
    let name = format!("{sha256}|{}|{}", mime.code(), canonical_json(metadata));
    Uuid::new_v5(&MEDIA_NAMESPACE, name.as_bytes())
}
