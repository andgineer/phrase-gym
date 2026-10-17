use std::fmt::Write;

use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

// Sorted keys and compact separators make semantically equal documents byte-identical.
pub fn canonical_json(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .and_then(|document| serde_json::to_string(&document))
        .unwrap_or_else(|_| unreachable!("contract values always serialize"))
}

pub fn fingerprint(value: &impl Serialize) -> String {
    sha256_hex(canonical_json(value).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_documents_ignore_key_order_but_keep_array_order() {
        let first: serde_json::Value = serde_json::from_str(r#"{"b": [1, 2], "a": "é"}"#).unwrap();
        let reordered: serde_json::Value =
            serde_json::from_str(r#"{"a": "é", "b": [1, 2]}"#).unwrap();
        let swapped: serde_json::Value =
            serde_json::from_str(r#"{"a": "é", "b": [2, 1]}"#).unwrap();
        assert_eq!(canonical_json(&first), r#"{"a":"é","b":[1,2]}"#);
        assert_eq!(fingerprint(&first), fingerprint(&reordered));
        assert_ne!(fingerprint(&first), fingerprint(&swapped));
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
