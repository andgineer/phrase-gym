use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use directories::{BaseDirs, ProjectDirs};

pub const BUNDLE_IDENTIFIER: &str = "io.github.andgineer.phrase-gym-coach";
const QUALIFIER: &str = "io.github";
const ORGANIZATION: &str = "andgineer";
const APPLICATION: &str = "Phrase Gym Coach";

#[derive(Debug, thiserror::Error)]
pub enum PathsError {
    #[error("the operating system reports no home directory for this user")]
    NoHome,
    #[error("cannot create {path}: {source}")]
    Create { path: PathBuf, source: io::Error },
    #[error("cannot restrict {path} to the current user: {source}")]
    Restrict { path: PathBuf, source: io::Error },
}

// Per-user locations only: Coach never writes configuration or cache beside its installed binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoachPaths {
    pub config: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
    pub state: PathBuf,
}

impl CoachPaths {
    pub fn for_current_user() -> Result<Self, PathsError> {
        let project =
            ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or(PathsError::NoHome)?;
        let base = BaseDirs::new().ok_or(PathsError::NoHome)?;
        Ok(Self {
            config: project.config_dir().to_path_buf(),
            cache: project.cache_dir().to_path_buf(),
            logs: log_directory(&project, &base),
            state: project.data_local_dir().to_path_buf(),
        })
    }

    pub fn instance_lock(&self) -> PathBuf {
        self.config.join("coach.lock")
    }

    pub fn attempts(&self) -> PathBuf {
        self.cache.join("attempts")
    }

    pub fn codex_home(&self) -> PathBuf {
        self.state.join("codex-home")
    }

    // Codex refuses a missing CODEX_HOME, and that folder holds Coach's Codex login; Windows
    // per-user local data already admits only the user, SYSTEM and administrators.
    pub fn ensure(&self) -> Result<(), PathsError> {
        let codex_home = self.codex_home();
        for directory in [
            &self.config,
            &self.cache,
            &self.logs,
            &self.state,
            &codex_home,
        ] {
            std::fs::create_dir_all(directory).map_err(|source| PathsError::Create {
                path: directory.clone(),
                source,
            })?;
        }
        #[cfg(unix)]
        std::fs::set_permissions(&codex_home, std::fs::Permissions::from_mode(0o700)).map_err(
            |source| PathsError::Restrict {
                path: codex_home.clone(),
                source,
            },
        )?;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn log_directory(_project: &ProjectDirs, base: &BaseDirs) -> PathBuf {
    base.home_dir().join("Library/Logs").join(APPLICATION)
}

#[cfg(not(target_os = "macos"))]
fn log_directory(project: &ProjectDirs, _base: &BaseDirs) -> PathBuf {
    project.data_local_dir().join("logs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_location_is_per_user_and_outside_the_installed_binary() {
        let paths = CoachPaths::for_current_user().unwrap();
        let home = BaseDirs::new().unwrap().home_dir().to_path_buf();
        let executable = std::env::current_exe().unwrap();
        let install = executable.parent().unwrap();
        for directory in [&paths.config, &paths.cache, &paths.logs, &paths.state] {
            assert!(directory.is_absolute(), "{}", directory.display());
            assert!(directory.starts_with(&home), "{}", directory.display());
            assert!(!directory.starts_with(install), "{}", directory.display());
            let text = directory.to_string_lossy().to_lowercase();
            assert!(text.contains("phrase") && text.contains("coach"), "{text}");
        }
        assert!(paths.instance_lock().starts_with(&paths.config));
        assert!(paths.attempts().starts_with(&paths.cache));
        assert!(paths.codex_home().starts_with(&paths.state));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn uses_the_macos_library_conventions() {
        let paths = CoachPaths::for_current_user().unwrap();
        let home = BaseDirs::new().unwrap().home_dir().to_path_buf();
        assert!(
            paths
                .config
                .starts_with(home.join("Library/Application Support"))
        );
        assert!(paths.cache.starts_with(home.join("Library/Caches")));
        assert_eq!(paths.logs, home.join("Library/Logs/Phrase Gym Coach"));
    }

    #[test]
    fn ensure_creates_missing_directories_with_spaces_and_unicode() {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join("Ünï Ђ user");
        let paths = CoachPaths {
            config: base.join("config"),
            cache: base.join("cache"),
            logs: base.join("logs"),
            state: base.join("state"),
        };
        paths.ensure().unwrap();
        paths.ensure().unwrap();
        assert!(paths.logs.is_dir() && paths.state.is_dir());
    }

    #[test]
    fn ensure_prepares_the_codex_home_for_its_owner_only() {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join("Ünï Ђ user");
        let paths = CoachPaths {
            config: base.join("config"),
            cache: base.join("cache"),
            logs: base.join("logs"),
            state: base.join("state"),
        };
        paths.ensure().unwrap();
        assert!(paths.codex_home().is_dir());
        #[cfg(unix)]
        {
            let mode =
                |path: PathBuf| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode(paths.codex_home()), 0o700);
            std::fs::set_permissions(paths.codex_home(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
            paths.ensure().unwrap();
            assert_eq!(mode(paths.codex_home()), 0o700);
        }
    }
}
