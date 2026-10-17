use std::path::{Path, PathBuf};

use phrase_gym_core::storage::migrations::{APPLICATION_ID, MIGRATIONS, Migration, migrate};
use phrase_gym_core::storage::{DATABASE_FILE, Database, StorageError};
use rusqlite::Connection;

fn schema(path: &Path) -> Vec<(String, String)> {
    let connection = Connection::open(path).unwrap();
    let mut statement = connection
        .prepare("SELECT name, sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY type, name")
        .unwrap();
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn pragma(path: &Path, name: &str) -> i64 {
    Connection::open(path)
        .unwrap()
        .query_row(&format!("PRAGMA {name}"), [], |row| row.get(0))
        .unwrap()
}

fn table_names(path: &Path) -> Vec<String> {
    schema(path)
        .into_iter()
        .filter(|(_, sql)| sql.starts_with("CREATE TABLE"))
        .map(|(name, _)| name)
        .collect()
}

fn chain_with(second: &'static str) -> Vec<Migration> {
    vec![
        Migration {
            revision: 1,
            sql: MIGRATIONS[0].sql,
        },
        Migration {
            revision: 2,
            sql: second,
        },
    ]
}

const BROKEN_SECOND: &str = "CREATE TABLE partial_step (id TEXT PRIMARY KEY) STRICT;\
                             INSERT INTO table_that_does_not_exist VALUES (1);";
const FIXED_SECOND: &str = "CREATE TABLE partial_step (id TEXT PRIMARY KEY) STRICT;";

fn database_file(directory: &Path) -> PathBuf {
    directory.join(DATABASE_FILE)
}

#[test]
fn fresh_initialization_applies_the_chain_once_with_integrity_settings() {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::open(directory.path()).unwrap();
    let path = database.path().to_owned();
    assert_eq!(pragma(&path, "user_version"), 1);
    assert_eq!(pragma(&path, "application_id"), APPLICATION_ID);
    assert!(table_names(&path).contains(&"target".to_owned()));
    let connection = Connection::open(&path).unwrap();
    let mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
    let violations: i64 = connection
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(violations, 0);
    database
        .write(|transaction| {
            let enforced: i64 =
                transaction.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
            assert_eq!(enforced, 1);
            let orphan = transaction.execute(
                "INSERT INTO target_revision VALUES ('missing', 1, '{}', 'x', 'now')",
                [],
            );
            assert!(orphan.is_err());
            Ok::<_, rusqlite::Error>(())
        })
        .unwrap();
}

#[test]
fn repeated_startup_changes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    drop(Database::open(directory.path()).unwrap());
    let path = database_file(directory.path());
    let before = schema(&path);
    drop(Database::open(directory.path()).unwrap());
    drop(Database::open(directory.path()).unwrap());
    assert_eq!(schema(&path), before);
    assert_eq!(pragma(&path, "user_version"), 1);
}

#[test]
fn reads_use_a_query_only_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::open(directory.path()).unwrap();
    let attempted = database.read(|transaction| {
        transaction.execute(
            "INSERT INTO lesson VALUES ('l', 'n', 's', 'en', 'Title')",
            [],
        )
    });
    assert!(attempted.is_err());
    let count: i64 = database
        .read(|transaction| {
            transaction.query_row("SELECT count(*) FROM lesson", [], |row| row.get(0))
        })
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn an_interrupted_upgrade_keeps_the_previous_revision_and_completes_on_retry() {
    let directory = tempfile::tempdir().unwrap();
    drop(Database::open(directory.path()).unwrap());
    let path = database_file(directory.path());
    let mut connection = Connection::open(&path).unwrap();
    assert!(migrate(&mut connection, &chain_with(BROKEN_SECOND)).is_err());
    drop(connection);
    assert_eq!(pragma(&path, "user_version"), 1);
    assert!(!table_names(&path).contains(&"partial_step".to_owned()));
    let mut connection = Connection::open(&path).unwrap();
    migrate(&mut connection, &chain_with(FIXED_SECOND)).unwrap();
    drop(connection);
    assert_eq!(pragma(&path, "user_version"), 2);
    assert!(table_names(&path).contains(&"partial_step".to_owned()));
}

#[test]
fn an_interrupted_initialization_leaves_an_empty_database_that_initializes_later() {
    let directory = tempfile::tempdir().unwrap();
    let path = database_file(directory.path());
    let mut connection = Connection::open(&path).unwrap();
    assert!(migrate(&mut connection, &chain_with(BROKEN_SECOND)).is_err());
    drop(connection);
    assert_eq!(pragma(&path, "user_version"), 0);
    assert_eq!(pragma(&path, "application_id"), 0);
    assert!(table_names(&path).is_empty());
    drop(Database::open(directory.path()).unwrap());
    assert_eq!(pragma(&path, "user_version"), 1);
}

fn assert_refused_untouched(directory: &Path, expected: &str) {
    let path = database_file(directory);
    let before = std::fs::read(&path).unwrap();
    let error = Database::open(directory).unwrap_err();
    assert!(
        matches!(&error, StorageError::UnsupportedSchema(message) if message.contains(expected)),
        "{error}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let wal = path.with_file_name(format!("{DATABASE_FILE}-wal"));
    assert!(!wal.exists());
}

#[test]
fn a_newer_schema_is_refused_without_writing() {
    let directory = tempfile::tempdir().unwrap();
    let connection = Connection::open(database_file(directory.path())).unwrap();
    connection
        .execute_batch(&format!(
            "CREATE TABLE future (id TEXT); PRAGMA application_id = {APPLICATION_ID}; PRAGMA user_version = 99;"
        ))
        .unwrap();
    drop(connection);
    assert_refused_untouched(
        directory.path(),
        "revision 99 was written by a newer release",
    );
}

#[test]
fn foreign_and_unversioned_databases_are_refused_without_writing() {
    let foreign = tempfile::tempdir().unwrap();
    Connection::open(database_file(foreign.path()))
        .unwrap()
        .execute_batch("CREATE TABLE notes (id INTEGER PRIMARY KEY); PRAGMA user_version = 1;")
        .unwrap();
    assert_refused_untouched(foreign.path(), "not a Phrase Gym database");

    let unversioned = tempfile::tempdir().unwrap();
    Connection::open(database_file(unversioned.path()))
        .unwrap()
        .execute_batch(&format!(
            "CREATE TABLE target (id TEXT); PRAGMA application_id = {APPLICATION_ID};"
        ))
        .unwrap();
    assert_refused_untouched(unversioned.path(), "not a Phrase Gym database");

    let not_sqlite = tempfile::tempdir().unwrap();
    std::fs::write(
        database_file(not_sqlite.path()),
        b"plain text that is not a database file at all, long enough to look like a header",
    )
    .unwrap();
    assert_refused_untouched(not_sqlite.path(), "not a SQLite database");
}
