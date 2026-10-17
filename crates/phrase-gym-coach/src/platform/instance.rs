use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("another Phrase Gym Coach is already running for this user")]
    AlreadyRunning,
    #[error("cannot use the instance lock {path}: {source}")]
    Lock { path: PathBuf, source: io::Error },
}

// The OS releases the lock when the holding process exits, including after a crash, so a stale
// lock file never blocks the next launch.
#[derive(Debug)]
pub struct InstanceGuard {
    _file: File,
}

impl InstanceGuard {
    pub fn acquire(lock: &Path) -> Result<Self, InstanceError> {
        let failure = |source| InstanceError::Lock {
            path: lock.to_path_buf(),
            source,
        };
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(lock)
            .map_err(failure)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(InstanceError::AlreadyRunning),
            Err(TryLockError::Error(source)) => Err(failure(source)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_instance_is_refused_until_the_first_releases_the_lock() {
        let root = tempfile::tempdir().unwrap();
        let lock = root.path().join("Ünï Ђ coach.lock");
        let first = InstanceGuard::acquire(&lock).unwrap();
        assert!(matches!(
            InstanceGuard::acquire(&lock),
            Err(InstanceError::AlreadyRunning)
        ));
        drop(first);
        InstanceGuard::acquire(&lock).unwrap();
    }

    #[test]
    fn reports_an_unusable_lock_location() {
        let root = tempfile::tempdir().unwrap();
        let lock = root.path().join("missing").join("coach.lock");
        assert!(matches!(
            InstanceGuard::acquire(&lock),
            Err(InstanceError::Lock { .. })
        ));
    }
}
