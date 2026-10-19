pub(crate) mod connection;
pub mod migrations;

use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use rusqlite::{Connection, Transaction, TransactionBehavior};
use thiserror::Error;

pub const DATABASE_FILE: &str = "phrase-gym.sqlite3";
pub const RECOVERY_PENDING: &str = ".recovery-pending";
pub const SNAPSHOT_MANIFEST: &str = "snapshot.json";
const IDLE_READERS: usize = 8;

pub fn recovery_reservation(directory: &Path) -> PathBuf {
    let mut name = directory.file_name().unwrap_or_default().to_os_string();
    name.push(RECOVERY_PENDING);
    directory.with_file_name(name)
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    UnsupportedSchema(String),
    #[error("stored record is unreadable: {0}")]
    CorruptRecord(String),
}

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    writer: Mutex<Connection>,
    readers: Mutex<Vec<Connection>>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn resolve_directory(directory: &Path) -> io::Result<PathBuf> {
    match std::fs::symlink_metadata(directory) {
        Ok(_) => directory.canonicalize(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = directory
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let mut resolved = resolve_directory(parent)?;
            if directory.components().next_back() == Some(Component::ParentDir) {
                resolved.pop();
                return Ok(resolved);
            }
            resolved.push(directory.file_name().ok_or(error)?);
            // Cancelling missing components can expose an existing symlink again.
            match std::fs::symlink_metadata(&resolved) {
                Ok(_) => resolved.canonicalize(),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(resolved),
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

impl Database {
    pub fn open(data_dir: &Path) -> Result<Self, StorageError> {
        // Recovery reserves the canonical destination before its internal marker can be written.
        let recovery_dir = resolve_directory(data_dir)?;
        if recovery_dir.join(RECOVERY_PENDING).try_exists()?
            || recovery_dir.join(SNAPSHOT_MANIFEST).try_exists()?
            || recovery_reservation(&recovery_dir).try_exists()?
        {
            return Err(StorageError::UnsupportedSchema(
                "snapshot or incomplete recovery directory; restore to a fresh directory first"
                    .to_owned(),
            ));
        }
        std::fs::create_dir_all(data_dir)?;
        let path = data_dir.join(DATABASE_FILE);
        migrations::recognize_file(&path, migrations::MIGRATIONS)?;
        let mut writer = connection::open_writer(&path)?;
        migrations::migrate(&mut writer, migrations::MIGRATIONS)?;
        Ok(Self {
            path,
            writer: Mutex::new(writer),
            readers: Mutex::new(Vec::new()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // One serialized writer keeps write transactions short and ordered; callers must not block inside.
    pub fn write<T, E>(&self, work: impl FnOnce(&Transaction<'_>) -> Result<T, E>) -> Result<T, E>
    where
        E: From<rusqlite::Error>,
    {
        let mut connection = locked(&self.writer);
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let value = work(&transaction)?;
        transaction.commit()?;
        Ok(value)
    }

    // Reads see one consistent snapshot on a query-only connection, so they cannot write by accident.
    pub fn read<T, E>(&self, work: impl FnOnce(&Transaction<'_>) -> Result<T, E>) -> Result<T, E>
    where
        E: From<rusqlite::Error>,
    {
        let pooled = locked(&self.readers).pop();
        let mut connection = match pooled {
            Some(connection) => connection,
            None => connection::open_reader(&self.path)?,
        };
        let result = {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
            work(&transaction)
        };
        let mut idle = locked(&self.readers);
        if idle.len() < IDLE_READERS {
            idle.push(connection);
        }
        result
    }
}
