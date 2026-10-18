use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::hashing::sha256_hex;

#[derive(Debug)]
pub struct MediaStore {
    root: PathBuf,
    staging: PathBuf,
}

pub struct StagedFile {
    path: PathBuf,
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl MediaStore {
    pub fn open(data_dir: &Path) -> std::io::Result<Self> {
        let store = Self {
            root: data_dir.join("media"),
            staging: data_dir.join("staging"),
        };
        std::fs::create_dir_all(&store.root)?;
        std::fs::create_dir_all(&store.staging)?;
        Ok(store)
    }

    fn path(&self, sha256: &str) -> PathBuf {
        self.root.join(sha256)
    }

    pub fn read(&self, sha256: &str) -> std::io::Result<Vec<u8>> {
        std::fs::read(self.path(sha256))
    }

    pub fn available(&self, sha256: &str, byte_length: i64) -> bool {
        self.read(sha256).is_ok_and(|bytes| {
            i64::try_from(bytes.len()) == Ok(byte_length) && sha256_hex(&bytes) == sha256
        })
    }

    pub fn stage(&self, bytes: &[u8]) -> std::io::Result<StagedFile> {
        let staged = StagedFile {
            path: self.staging.join(format!("{}.part", Uuid::new_v4())),
        };
        let mut file = File::create(&staged.path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(staged)
    }

    // The rename is atomic, so readers see either no original or the complete verified bytes.
    pub fn promote(&self, staged: &StagedFile, sha256: &str) -> std::io::Result<()> {
        std::fs::rename(&staged.path, self.path(sha256))?;
        sync_directory(&self.root)
    }

    pub fn stored_hashes(&self) -> std::io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        names.sort();
        Ok(names)
    }
}

#[cfg(unix)]
fn sync_directory(directory: &Path) -> std::io::Result<()> {
    File::open(directory)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_: &Path) -> std::io::Result<()> {
    Ok(())
}
