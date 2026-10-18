use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::error::CoreError;
use crate::hashing::sha256_hex;

pub(super) const FORMAT: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub format: u32,
    pub app_version: String,
    pub schema_revision: i64,
    pub database: Digest,
    pub media: BTreeMap<String, Digest>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Digest {
    pub sha256: String,
    pub byte_length: u64,
}

impl Digest {
    pub(super) fn file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 65_536];
        let mut byte_length = 0;
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
            byte_length += count as u64;
        }
        let sha256 = hasher
            .finalize()
            .iter()
            .fold(String::with_capacity(64), |mut text, byte| {
                let _ = write!(text, "{byte:02x}");
                text
            });
        Ok(Self {
            sha256,
            byte_length,
        })
    }

    pub(super) fn of(bytes: &[u8]) -> Self {
        Self {
            sha256: sha256_hex(bytes),
            byte_length: bytes.len() as u64,
        }
    }

    pub(super) fn check(&self, bytes: &[u8]) -> Result<(), CoreError> {
        if *self != Self::of(bytes) {
            return Err(CoreError::Invalid(
                "Recovery file checksum or length mismatch".to_owned(),
            ));
        }
        Ok(())
    }
}
