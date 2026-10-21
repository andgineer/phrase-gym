use std::path::Path;

use rusqlite::{Connection, ErrorCode, TransactionBehavior};

use super::{StorageError, connection};

pub struct Migration {
    pub revision: i64,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        revision: 1,
        sql: include_str!("../../migrations/0001_initial.sql"),
    },
    Migration {
        revision: 2,
        sql: include_str!("../../migrations/0002_practice.sql"),
    },
    Migration {
        revision: 3,
        sql: include_str!("../../migrations/0003_profiles.sql"),
    },
    Migration {
        revision: 4,
        sql: include_str!("../../migrations/0004_inbox.sql"),
    },
    Migration {
        revision: 5,
        sql: include_str!("../../migrations/0005_preparation_guide.sql"),
    },
    Migration {
        revision: 6,
        sql: include_str!("../../migrations/0006_working_set_guide.sql"),
    },
];

// Marks the file as a Phrase Gym database so a foreign SQLite file is never mistaken for an empty one.
pub const APPLICATION_ID: i64 = 0x5047_5359;

enum Recognized {
    Fresh,
    At(i64),
}

pub fn latest_revision(chain: &[Migration]) -> i64 {
    chain.last().map_or(0, |migration| migration.revision)
}

// Closing a writable connection checkpoints a hot WAL into the main file, so a file that may be refused
// is recognized through a read-only connection before any writer opens it.
pub fn recognize_file(path: &Path, chain: &[Migration]) -> Result<(), StorageError> {
    if !path.try_exists()? {
        return Ok(());
    }
    recognize(&connection::open_inspector(path)?, chain)?;
    Ok(())
}

pub fn migrate(connection: &mut Connection, chain: &[Migration]) -> Result<(), StorageError> {
    assert!(
        chain
            .iter()
            .zip(1..)
            .all(|(migration, expected)| migration.revision == expected),
        "migration revisions must count up from 1"
    );
    connection
        .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let applied = match recognize(&transaction, chain)? {
        Recognized::Fresh => 0,
        Recognized::At(revision) => revision,
    };
    let latest = latest_revision(chain);
    if applied < latest {
        for migration in chain
            .iter()
            .filter(|migration| migration.revision > applied)
        {
            transaction.execute_batch(migration.sql)?;
        }
        transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
        transaction.pragma_update(None, "user_version", latest)?;
    }
    transaction.commit()?;
    Ok(())
}

fn recognize(connection: &Connection, chain: &[Migration]) -> Result<Recognized, StorageError> {
    let read = |pragma: &str| -> Result<i64, StorageError> {
        connection
            .query_row(&format!("PRAGMA {pragma}"), [], |row| row.get(0))
            .map_err(|error| match error.sqlite_error_code() {
                Some(ErrorCode::NotADatabase) => {
                    StorageError::UnsupportedSchema("the file is not a SQLite database".to_owned())
                }
                _ => StorageError::Sqlite(error),
            })
    };
    let application = read("application_id")?;
    let revision = read("user_version")?;
    let objects: i64 = connection.query_row(
        "SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    )?;
    let latest = latest_revision(chain);
    match (application, revision) {
        (0, 0) if objects == 0 => Ok(Recognized::Fresh),
        (APPLICATION_ID, 1..) if revision <= latest => Ok(Recognized::At(revision)),
        (APPLICATION_ID, 1..) => Err(StorageError::UnsupportedSchema(format!(
            "database schema revision {revision} was written by a newer release; \
             this build supports revisions up to {latest}"
        ))),
        _ => Err(StorageError::UnsupportedSchema(
            "the file is not a Phrase Gym database with a recognized schema revision".to_owned(),
        )),
    }
}
