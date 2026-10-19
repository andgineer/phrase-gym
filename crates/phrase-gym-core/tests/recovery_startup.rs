#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;

use phrase_gym_core::storage::{DATABASE_FILE, Database, StorageError, recovery_reservation};

fn assert_reserved_untouched(path: &Path, destination: &Path) {
    let reservation = recovery_reservation(destination);
    let before = std::fs::read(&reservation).unwrap();
    let result = Database::open(path);
    let rejected = matches!(result, Err(StorageError::UnsupportedSchema(_)));
    assert!(!destination.join(DATABASE_FILE).exists());
    assert_eq!(std::fs::read(&reservation).unwrap(), before);
    assert!(rejected, "reserved recovery destination must be refused");
}

#[test]
fn relative_reservations_refuse_existing_and_not_yet_created_destinations() {
    let root = tempfile::tempdir_in(".").unwrap();
    let parent = root.path().canonicalize().unwrap();
    let working_directory = std::env::current_dir().unwrap().canonicalize().unwrap();
    for exists in [false, true] {
        let name = if exists { "existing" } else { "not-created" };
        let destination = parent.join(name);
        std::fs::write(recovery_reservation(&destination), b"reserved").unwrap();
        if exists {
            std::fs::create_dir(&destination).unwrap();
        }
        let relative = destination.strip_prefix(&working_directory).unwrap();
        assert!(!relative.is_absolute());
        assert_reserved_untouched(relative, &destination);
        assert_eq!(destination.exists(), exists);
        if exists {
            assert_eq!(std::fs::read_dir(&destination).unwrap().count(), 0);
        }
    }
}

#[cfg(unix)]
#[test]
fn directory_and_parent_aliases_cannot_bypass_the_canonical_reservation() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().canonicalize().unwrap();
    let destination = parent.join("interrupted");
    let alias = parent.join("alias");
    let parent_alias = parent.join("parent-alias");
    std::fs::write(recovery_reservation(&destination), b"reserved").unwrap();
    std::fs::create_dir(&destination).unwrap();
    symlink(&destination, &alias).unwrap();
    symlink(&parent, &parent_alias).unwrap();
    for path in [&destination, &alias, &parent_alias.join("interrupted")] {
        assert_reserved_untouched(path, &destination);
        assert_eq!(std::fs::read_dir(&destination).unwrap().count(), 0);
    }
    let missing = parent.join("not-created");
    std::fs::write(recovery_reservation(&missing), b"reserved").unwrap();
    assert_reserved_untouched(&parent_alias.join("not-created"), &missing);
    assert!(!missing.exists());
    assert!(std::fs::symlink_metadata(&alias).unwrap().is_symlink());
}

#[test]
fn fresh_nested_and_existing_directories_remain_usable() {
    let root = tempfile::tempdir_in(".").unwrap();
    let directory = root.path().join("new-parent").join("live");
    let database = Database::open(&directory).unwrap();
    database
        .write::<_, StorageError>(|transaction| {
            transaction.execute(
                "INSERT INTO lesson VALUES ('lesson', 'test', '1', 'en', 'Retained')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    drop(database);
    let reopened = Database::open(&directory).unwrap();
    let title: String = reopened
        .read::<_, StorageError>(|transaction| {
            Ok(transaction.query_row("SELECT title FROM lesson", [], |row| row.get(0))?)
        })
        .unwrap();
    assert_eq!(title, "Retained");
}

#[test]
fn fresh_paths_can_cancel_missing_parent_components_and_reopen() {
    for absolute in [false, true] {
        let root = tempfile::tempdir_in(".").unwrap();
        for suffix in ["unused/../live", "unused/nested/../../other-live"] {
            let relative = root.path().join(suffix);
            let directory = if absolute {
                std::env::current_dir().unwrap().join(relative)
            } else {
                relative
            };
            drop(Database::open(&directory).unwrap());
            let database = Database::open(&directory).unwrap();
            assert!(database.path().is_file());
            assert_eq!(
                database.path().canonicalize().unwrap(),
                root.path()
                    .join(directory.file_name().unwrap())
                    .join(DATABASE_FILE)
                    .canonicalize()
                    .unwrap()
            );
        }
    }
}

#[test]
fn missing_parent_traversal_refuses_reservations_before_creating_directories() {
    let root = tempfile::tempdir().unwrap();
    for exists in [false, true] {
        let destination = root
            .path()
            .join(if exists { "existing" } else { "not-created" });
        std::fs::write(recovery_reservation(&destination), b"reserved").unwrap();
        if exists {
            std::fs::create_dir(&destination).unwrap();
        }
        let path = root
            .path()
            .join("unused/nested/../..")
            .join(destination.file_name().unwrap());
        assert_reserved_untouched(&path, &destination);
        assert!(!root.path().join("unused").exists());
        assert_eq!(destination.exists(), exists);
        if exists {
            assert_eq!(std::fs::read_dir(&destination).unwrap().count(), 0);
        }
    }
}

#[cfg(unix)]
#[test]
fn existing_symlink_parent_traversal_keeps_filesystem_semantics() {
    let root = tempfile::tempdir().unwrap();
    let target_parent = root.path().join("real");
    let target = target_parent.join("child");
    std::fs::create_dir_all(&target).unwrap();
    symlink(&target, root.path().join("alias")).unwrap();
    for exists in [false, true] {
        let destination = target_parent.join("reserved-српски");
        std::fs::write(recovery_reservation(&destination), b"reserved").unwrap();
        if exists {
            std::fs::create_dir(&destination).unwrap();
        }
        for prefix in ["alias/..", "unused/../alias/.."] {
            assert_reserved_untouched(
                &root
                    .path()
                    .join(prefix)
                    .join(destination.file_name().unwrap()),
                &destination,
            );
            assert_eq!(destination.exists(), exists);
            assert!(!root.path().join("unused").exists());
            assert!(!root.path().join(destination.file_name().unwrap()).exists());
        }
    }
    let fresh = root.path().join("unused/../alias/../live");
    drop(Database::open(&fresh).unwrap());
    assert!(target_parent.join("live").join(DATABASE_FILE).is_file());
    assert!(!root.path().join("live").exists());
}

#[cfg(unix)]
#[test]
fn unreserved_aliases_work_and_unresolvable_aliases_fail_without_creating_data() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    std::fs::create_dir(&live).unwrap();
    let alias = root.path().join("alias");
    symlink(&live, &alias).unwrap();
    drop(Database::open(&alias).unwrap());
    assert!(live.join(DATABASE_FILE).exists());

    let missing = root.path().join("missing");
    let dangling = root.path().join("dangling");
    symlink(&missing, &dangling).unwrap();
    let looping = root.path().join("looping");
    symlink(&looping, &looping).unwrap();
    for path in [
        &dangling,
        &dangling.join("child"),
        &looping,
        &root.path().join("unused/../dangling"),
        &root.path().join("unused/../dangling/../other-live"),
        &root.path().join("unused/../looping"),
    ] {
        assert!(matches!(Database::open(path), Err(StorageError::Io(_))));
    }
    assert!(!missing.exists());
    assert!(std::fs::symlink_metadata(&dangling).unwrap().is_symlink());
    assert!(std::fs::symlink_metadata(&looping).unwrap().is_symlink());
}
