mod connection;
pub mod migrations;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use rusqlite::{Connection, Transaction, TransactionBehavior};
use thiserror::Error;

pub const DATABASE_FILE: &str = "phrase-gym.sqlite3";
const IDLE_READERS: usize = 8;

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

impl Database {
    pub fn open(data_dir: &Path) -> Result<Self, StorageError> {
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
