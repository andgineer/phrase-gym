use std::fs::File;
use std::path::Path;
use std::sync::mpsc;

use phrase_gym_core::hashing::sha256_hex;
use phrase_gym_core::recovery::{backup, restore, verify};
use phrase_gym_core::storage::migrations::{MIGRATIONS, latest_revision, migrate};
use phrase_gym_core::storage::{
    DATABASE_FILE, Database, RECOVERY_PENDING, SNAPSHOT_MANIFEST, recovery_reservation,
};
use rusqlite::{Connection, params};

const WAV: &[u8] = include_bytes!("../../../fixtures/preservation/media/silence-100ms.wav");

fn add_media(connection: &Connection, directory: &Path, id: &str, bytes: &[u8]) {
    std::fs::create_dir_all(directory.join("media")).unwrap();
    let hash = sha256_hex(bytes);
    std::fs::write(directory.join("media").join(&hash), bytes).unwrap();
    connection
        .execute(
            "INSERT INTO reference_media VALUES (?1, ?2, ?3, 'audio/wav', '{}')",
            params![id, hash, i64::try_from(bytes.len()).unwrap()],
        )
        .unwrap();
}

fn seeded(directory: &Path) -> Database {
    let database = Database::open(directory).unwrap();
    let connection = Connection::open(database.path()).unwrap();
    add_media(&connection, directory, "original", WAV);
    connection
        .execute(
            "INSERT INTO lesson VALUES ('lesson', 'test', '1', 'sr', 'Synthetic')",
            [],
        )
        .unwrap();
    database
}

#[test]
fn live_wal_and_concurrent_transactions_produce_one_database_and_media_image() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    let database = seeded(&live);
    let initial_main = std::fs::read(database.path()).unwrap();
    assert!(live.join(format!("{DATABASE_FILE}-wal")).exists());
    let (pending, ready) = mpsc::channel();
    let (release, resume) = mpsc::channel();
    let writer_dir = live.clone();
    let writer = std::thread::spawn(move || {
        let mut connection = Connection::open(writer_dir.join(DATABASE_FILE)).unwrap();
        for index in 0..30 {
            let transaction = connection.transaction().unwrap();
            let id = format!("concurrent-{index}");
            let bytes = format!("synthetic immutable media {index}");
            add_media(&transaction, &writer_dir, &id, bytes.as_bytes());
            transaction
                .execute(
                    "INSERT INTO lesson VALUES (?1, 'writer', ?1, 'en', ?1)",
                    [&id],
                )
                .unwrap();
            if index == 0 {
                pending.send(()).unwrap();
                resume.recv().unwrap();
            }
            transaction.commit().unwrap();
            std::thread::yield_now();
        }
    });
    ready.recv().unwrap();
    let during_write = root.path().join("uncommitted-write");
    assert_eq!(backup(&live, &during_write).unwrap().media.len(), 1);
    release.send(()).unwrap();
    for index in 0..4 {
        let snapshot = root.path().join(format!("backup-{index}"));
        let manifest = backup(&live, &snapshot).unwrap();
        verify(&snapshot).unwrap();
        let connection = Connection::open(snapshot.join(DATABASE_FILE)).unwrap();
        let lessons: i64 = connection
            .query_row("SELECT count(*) FROM lesson", [], |r| r.get(0))
            .unwrap();
        assert_eq!(i64::try_from(manifest.media.len()).unwrap(), lessons);
        assert_eq!(std::fs::read(database.path()).unwrap(), initial_main);
    }
    writer.join().unwrap();
    let latest = root.path().join("latest");
    assert_eq!(backup(&live, &latest).unwrap().media.len(), 31);
}

#[test]
fn damaged_recording_or_foreign_key_state_never_becomes_a_complete_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    let database = seeded(&live);
    let connection = Connection::open(database.path()).unwrap();
    connection
        .execute_batch(
            "INSERT INTO target VALUES ('t', 'en', 'fixed_phrase', 'm', 'text', 1);
        INSERT INTO target_revision VALUES ('t', 1, '{}', 'fingerprint', 'now');
        INSERT INTO exposure VALUES ('e', 't', 1, '{}', 'now');",
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO learner_recording VALUES ('r', 't', 1, NULL, 'e', '{}', ?1, ?2, 'now')",
            params![WAV, sha256_hex(WAV)],
        )
        .unwrap();
    backup(&live, &root.path().join("valid")).unwrap();
    connection
        .execute("UPDATE learner_recording SET audio = X'00'", [])
        .unwrap();
    let damaged = root.path().join("damaged-recording");
    assert!(
        backup(&live, &damaged)
            .unwrap_err()
            .to_string()
            .contains("Recording checksum")
    );
    assert!(Database::open(&damaged).is_err());
    connection
        .execute("UPDATE learner_recording SET audio = ?1", [WAV])
        .unwrap();
    connection
        .pragma_update(None, "foreign_keys", "OFF")
        .unwrap();
    connection
        .execute(
            "INSERT INTO target_revision VALUES ('absent', 1, '{}', 'x', 'now')",
            [],
        )
        .unwrap();
    let orphan = root.path().join("orphan-record");
    assert!(
        backup(&live, &orphan)
            .unwrap_err()
            .to_string()
            .contains("integrity check")
    );
    assert!(Database::open(&orphan).is_err());
}

#[test]
fn a_newer_rust_source_is_refused_without_mutation_or_destination_creation() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    let database = seeded(&live);
    let connection = Connection::open(database.path()).unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    let main = std::fs::read(database.path()).unwrap();
    let wal_path = live.join(format!("{DATABASE_FILE}-wal"));
    let wal = std::fs::read(&wal_path).unwrap();
    let output = root.path().join("refused");
    assert!(
        backup(&live, &output)
            .unwrap_err()
            .to_string()
            .contains("newer release")
    );
    assert!(!output.exists());
    assert_eq!(std::fs::read(database.path()).unwrap(), main);
    assert_eq!(std::fs::read(wal_path).unwrap(), wal);
}

#[test]
fn rollback_restores_the_matching_rust_revision_and_media_without_touching_upgrade() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    std::fs::create_dir(&live).unwrap();
    let path = live.join(DATABASE_FILE);
    let mut first = Connection::open(&path).unwrap();
    migrate(&mut first, &MIGRATIONS[..1]).unwrap();
    add_media(&first, &live, "revision-one", WAV);
    first
        .execute(
            "INSERT INTO lesson VALUES ('l', 'n', 's', 'en', 'Before upgrade')",
            [],
        )
        .unwrap();
    let snapshot = root.path().join("before-upgrade");
    assert_eq!(backup(&live, &snapshot).unwrap().schema_revision, 1);
    drop(first);
    let upgraded = Database::open(&live).unwrap();
    let connection = Connection::open(upgraded.path()).unwrap();
    add_media(&connection, &live, "revision-two", b"revision two media");
    connection
        .execute("UPDATE lesson SET title = 'After upgrade'", [])
        .unwrap();
    let rollback = root.path().join("rollback");
    assert_eq!(restore(&snapshot, &rollback).unwrap().schema_revision, 1);
    let restored = Connection::open(rollback.join(DATABASE_FILE)).unwrap();
    let revision: i64 = restored
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(revision, 1);
    let title: String = restored
        .query_row("SELECT title FROM lesson", [], |r| r.get(0))
        .unwrap();
    assert_eq!(title, "Before upgrade");
    assert!(
        !rollback
            .join("media")
            .join(sha256_hex(b"revision two media"))
            .exists()
    );
    assert_eq!(
        std::fs::read(rollback.join("media").join(sha256_hex(WAV))).unwrap(),
        WAV
    );
    assert_eq!(
        backup(&live, &root.path().join("after-upgrade"))
            .unwrap()
            .schema_revision,
        latest_revision(MIGRATIONS)
    );
}

#[test]
fn missing_corrupt_or_incomplete_snapshots_are_rejected_before_destination_creation() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    let _database = seeded(&live);
    for damage in [
        "missing-media",
        "corrupt-media",
        "database",
        "manifest",
        "incomplete",
        "sidecar",
        "revision",
        "catalog",
        "extra-media",
        "format",
        "unknown-field",
    ] {
        let snapshot = root.path().join(damage);
        backup(&live, &snapshot).unwrap();
        let media = snapshot.join("media").join(sha256_hex(WAV));
        match damage {
            "missing-media" => std::fs::remove_file(media).unwrap(),
            "corrupt-media" => std::fs::write(media, b"damaged").unwrap(),
            "database" => std::fs::write(snapshot.join(DATABASE_FILE), b"damaged").unwrap(),
            "manifest" => std::fs::remove_file(snapshot.join(SNAPSHOT_MANIFEST)).unwrap(),
            "incomplete" => std::fs::write(snapshot.join(RECOVERY_PENDING), b"pending").unwrap(),
            "sidecar" => {
                std::fs::write(snapshot.join(format!("{DATABASE_FILE}-wal")), b"wal").unwrap();
            }
            "revision" | "catalog" | "extra-media" | "format" | "unknown-field" => {
                let path = snapshot.join(SNAPSHOT_MANIFEST);
                let mut manifest: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                match damage {
                    "revision" => manifest["schema_revision"] = serde_json::json!(1),
                    "catalog" => manifest["media"] = serde_json::json!({}),
                    "extra-media" => manifest["media"]["extra"] = manifest["database"].clone(),
                    "format" => manifest["format"] = serde_json::json!(99),
                    "unknown-field" => manifest["unexpected"] = serde_json::json!(true),
                    _ => unreachable!(),
                }
                std::fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        let output = root.path().join(format!("restore-{damage}"));
        assert!(restore(&snapshot, &output).is_err(), "{damage}");
        assert!(!output.exists(), "{damage}");
    }
}

#[test]
fn destinations_are_isolated_no_overwrite_and_partial_recovery_cannot_start() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    let _database = seeded(&live);
    let snapshot = root.path().join("snapshot");
    backup(&live, &snapshot).unwrap();
    let database_before = std::fs::read(snapshot.join(DATABASE_FILE)).unwrap();
    let manifest_before = std::fs::read(snapshot.join(SNAPSHOT_MANIFEST)).unwrap();
    verify(&snapshot).unwrap();
    verify(&snapshot).unwrap();
    assert_eq!(
        std::fs::read(snapshot.join(DATABASE_FILE)).unwrap(),
        database_before
    );
    assert_eq!(
        std::fs::read(snapshot.join(SNAPSHOT_MANIFEST)).unwrap(),
        manifest_before
    );
    assert_eq!(std::fs::read_dir(&snapshot).unwrap().count(), 3);
    assert!(Database::open(&snapshot).is_err());
    assert!(backup(&live, &live.join("nested")).is_err());
    assert!(backup(&live, &snapshot).is_err());
    assert!(restore(&snapshot, &live).is_err());
    assert!(restore(&snapshot, &snapshot.join("nested")).is_err());
    let empty = root.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    assert!(restore(&snapshot, &empty).is_err());
    let broken = root.path().join("broken-backup");
    std::fs::remove_file(live.join("media").join(sha256_hex(WAV))).unwrap();
    assert!(backup(&live, &broken).is_err());
    assert!(broken.join(RECOVERY_PENDING).exists());
    assert!(Database::open(&broken).is_err());
    assert!(verify(&broken).is_err());
    let crash = root.path().join("crash-before-directory-marker");
    File::create(recovery_reservation(&crash)).unwrap();
    std::fs::create_dir(&crash).unwrap();
    assert!(Database::open(&crash).is_err());
    assert!(!crash.join(DATABASE_FILE).exists());
}
