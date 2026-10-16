use rusqlite::{Connection, Transaction, TransactionBehavior};

use super::StorageError;

pub struct Migration {
    pub revision: &'static str,
    pub sql: &'static str,
}

// Revision identifiers continue the existing Alembic chain so legacy databases are recognized in place.
pub const MIGRATIONS: &[Migration] = &[Migration {
    revision: "0001",
    sql: include_str!("../../migrations/0001_foundation.sql"),
}];

enum Recognized {
    Empty,
    At(String),
}

pub fn migrate(connection: &mut Connection) -> Result<(), StorageError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let pending = match recognize(&transaction)? {
        Recognized::Empty => 0,
        Recognized::At(revision) => MIGRATIONS
            .iter()
            .position(|migration| migration.revision == revision)
            .map(|index| index + 1)
            .ok_or_else(|| {
                StorageError::UnsupportedSchema(format!(
                    "database schema revision {revision} is not supported by this build"
                ))
            })?,
    };
    if let Some(latest) = MIGRATIONS[pending..].last() {
        for migration in &MIGRATIONS[pending..] {
            transaction.execute_batch(migration.sql)?;
        }
        transaction.execute("DELETE FROM alembic_version", [])?;
        transaction.execute(
            "INSERT INTO alembic_version (version_num) VALUES (?1)",
            [latest.revision],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

fn recognize(transaction: &Transaction<'_>) -> Result<Recognized, StorageError> {
    let tables: i64 = transaction.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    )?;
    if tables == 0 {
        return Ok(Recognized::Empty);
    }
    let unrecognized =
        || StorageError::UnsupportedSchema("database has no recognized schema revision".to_owned());
    let has_version_table: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'alembic_version')",
        [],
        |row| row.get(0),
    )?;
    if !has_version_table {
        return Err(unrecognized());
    }
    let mut statement = transaction.prepare("SELECT version_num FROM alembic_version")?;
    let mut revisions = statement.query_map([], |row| row.get::<_, String>(0))?;
    let first = revisions.next().transpose()?.ok_or_else(unrecognized)?;
    if revisions.next().transpose()?.is_some() {
        return Err(unrecognized());
    }
    Ok(Recognized::At(first))
}
