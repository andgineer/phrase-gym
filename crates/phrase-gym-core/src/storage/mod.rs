mod connection;
pub mod migrations;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use rusqlite::{Connection, Transaction, TransactionBehavior};
use thiserror::Error;

pub const DATABASE_FILE: &str = "phrase-gym.sqlite3";

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
}

impl Database {
    pub fn open(data_dir: &Path) -> Result<Self, StorageError> {
        std::fs::create_dir_all(data_dir)?;
        let path = data_dir.join(DATABASE_FILE);
        let mut writer = connection::open(&path)?;
        migrations::migrate(&mut writer)?;
        Ok(Self {
            path,
            writer: Mutex::new(writer),
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
        let mut connection = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let value = work(&transaction)?;
        transaction.commit()?;
        Ok(value)
    }
}
