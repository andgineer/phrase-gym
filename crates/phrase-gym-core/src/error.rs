use thiserror::Error;

use crate::storage::StorageError;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl From<rusqlite::Error> for CoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::Sqlite(error))
    }
}
