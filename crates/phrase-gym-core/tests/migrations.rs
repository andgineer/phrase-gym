use std::path::{Path, PathBuf};

use phrase_gym_core::storage::{DATABASE_FILE, Database, StorageError};
use rusqlite::Connection;

fn legacy_schema(revision: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../fixtures/preservation/schema/alembic-{revision}.sql"
    ));
    std::fs::read_to_string(path).unwrap()
}

fn normalized(sql: &str) -> String {
    let collapsed = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut output = String::new();
    for character in collapsed.chars() {
        if matches!(character, '(' | ')' | ',') && output.ends_with(' ') {
            output.pop();
        }
        if character == ' ' && output.ends_with(['(', ')', ',']) {
            continue;
        }
        output.push(character);
    }
    output
}

fn schema(connection: &Connection) -> Vec<(String, String, String)> {
    let mut statement = connection
        .prepare(
            "SELECT type, name, sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY type, name",
        )
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                normalized(&row.get::<_, String>(2)?),
            ))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn legacy_database(directory: &Path, revision: &str) -> PathBuf {
    let path = directory.join(DATABASE_FILE);
    Connection::open(&path)
        .unwrap()
        .execute_batch(&legacy_schema(revision))
        .unwrap();
    path
}

fn revision(path: &Path) -> Vec<String> {
    let connection = Connection::open(path).unwrap();
    let mut statement = connection
        .prepare("SELECT version_num FROM alembic_version")
        .unwrap();
    statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn fresh_initialization_matches_the_legacy_foundation_schema() {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::open(directory.path()).unwrap();
    let expected = Connection::open_in_memory().unwrap();
    expected.execute_batch(&legacy_schema("0001")).unwrap();
    assert_eq!(
        schema(&Connection::open(database.path()).unwrap()),
        schema(&expected)
    );
    assert_eq!(revision(database.path()), ["0001"]);
    drop(database);
    Database::open(directory.path()).unwrap();
    assert_eq!(revision(&directory.path().join(DATABASE_FILE)), ["0001"]);
}

#[test]
fn a_legacy_foundation_database_is_recognized_in_place() {
    let directory = tempfile::tempdir().unwrap();
    let path = legacy_database(directory.path(), "0001");
    let before = schema(&Connection::open(&path).unwrap());
    Database::open(directory.path()).unwrap();
    assert_eq!(schema(&Connection::open(&path).unwrap()), before);
    assert_eq!(revision(&path), ["0001"]);
}

#[test]
fn unsupported_revisions_are_rejected_before_any_write() {
    for legacy in ["0002", "0005"] {
        let directory = tempfile::tempdir().unwrap();
        let path = legacy_database(directory.path(), legacy);
        let before = schema(&Connection::open(&path).unwrap());
        let error = Database::open(directory.path()).unwrap_err();
        assert!(
            matches!(&error, StorageError::UnsupportedSchema(message) if message.contains(legacy)),
            "{error}"
        );
        assert_eq!(schema(&Connection::open(&path).unwrap()), before);
        assert_eq!(revision(&path), [legacy]);
    }
}

#[test]
fn databases_without_a_recognized_revision_are_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(DATABASE_FILE);
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("CREATE TABLE notes (id INTEGER PRIMARY KEY)")
        .unwrap();
    assert!(matches!(
        Database::open(directory.path()),
        Err(StorageError::UnsupportedSchema(_))
    ));
    connection
        .execute_batch(
            "CREATE TABLE alembic_version (version_num VARCHAR(32) NOT NULL PRIMARY KEY);\
             INSERT INTO alembic_version VALUES ('0001'), ('0002');",
        )
        .unwrap();
    assert!(matches!(
        Database::open(directory.path()),
        Err(StorageError::UnsupportedSchema(_))
    ));
}
