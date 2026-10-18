use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::media::SettingValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct RecordingInput {
    pub id: Uuid,
    pub target_id: Uuid,
    #[cfg_attr(feature = "openapi", schema(minimum = 1))]
    pub revision: i64,
    #[serde(default)]
    pub attempt_id: Option<Uuid>,
    #[serde(default)]
    pub exposure_id: Option<Uuid>,
    #[serde(default)]
    pub capture_event_id: Option<Uuid>,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub captured_at: String,
    #[cfg_attr(feature = "openapi", schema(min_length = 1, max_length = 100))]
    pub mime_type: String,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(minimum = 0))]
    pub duration_ms: Option<i64>,
    pub complete: bool,
    #[serde(default)]
    pub capture_settings: BTreeMap<String, SettingValue>,
    #[cfg_attr(feature = "openapi", schema(min_length = 1, max_length = 17_000_000))]
    pub audio_base64: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RecordingReceipt {
    pub id: Uuid,
    pub sha256: String,
    pub byte_length: i64,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub captured_at: String,
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub received_at: String,
    pub complete: bool,
}
