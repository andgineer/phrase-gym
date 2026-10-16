use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum ExposureKind {
    Answer,
    Explanation,
    Reference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ExposureInput {
    pub id: Uuid,
    pub target_id: Uuid,
    #[cfg_attr(feature = "openapi", schema(minimum = 1))]
    pub revision: i64,
    pub language: Language,
    pub kind: ExposureKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExposureReceipt {
    pub id: Uuid,
    pub received_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_fields_and_unlisted_kinds() {
        let valid = serde_json::json!({
            "id": "6a1f3c1e-0d7b-4e43-9a51-3f3e2f6b8a01",
            "target_id": "0b9d2a4f-7f5e-4c2b-8a3d-1e6f9c0b7d12",
            "revision": 1,
            "language": "sr",
            "kind": "reference",
        });
        let parsed: ExposureInput = serde_json::from_value(valid.clone()).unwrap();
        assert_eq!(parsed.language, Language::Sr);
        assert_eq!(parsed.kind, ExposureKind::Reference);
        let mut extra = valid.clone();
        extra["source"] = "lookup".into();
        assert!(serde_json::from_value::<ExposureInput>(extra).is_err());
        let mut unlisted = valid;
        unlisted["kind"] = "hint".into();
        assert!(serde_json::from_value::<ExposureInput>(unlisted).is_err());
    }
}
