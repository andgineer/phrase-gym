use std::path::{Path, PathBuf};

use phrase_gym_coach::adapter::CliAdapter;
use phrase_gym_coach::adapter::environment::HostEnvironment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    MacOs,
    Windows,
}

impl Platform {
    pub const CURRENT: Self = if cfg!(windows) {
        Self::Windows
    } else {
        Self::MacOs
    };

    // The folder both CLIs resolve `~` against; Windows normally has no HOME at all.
    fn home_variable(self) -> &'static str {
        match self {
            Self::MacOs => "HOME",
            Self::Windows => "USERPROFILE",
        }
    }
}

pub fn profile_home(platform: Platform, host: &HostEnvironment) -> Option<PathBuf> {
    host.get(platform.home_variable())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

// Most sensitive first. Claude keeps its login in `.claude/.credentials.json` on Windows and in
// the Keychain on macOS, falling back to that file when the Keychain refuses the write.
pub fn credential_candidates(
    adapter: CliAdapter,
    platform: Platform,
    host: &HostEnvironment,
    codex_home: &Path,
) -> Result<Vec<PathBuf>, String> {
    match adapter {
        CliAdapter::Codex => Ok(vec![codex_home.join("auth.json")]),
        CliAdapter::Claude => {
            let home = profile_home(platform, host).ok_or_else(|| {
                format!(
                    "{} is not set, so Claude's credential files cannot be located",
                    platform.home_variable()
                )
            })?;
            Ok(vec![
                home.join(".claude").join(".credentials.json"),
                home.join(".claude.json"),
            ])
        }
    }
}

pub fn existing_target(candidates: &[PathBuf]) -> Result<PathBuf, String> {
    candidates
        .iter()
        .find(|candidate| candidate.is_file())
        .cloned()
        .ok_or_else(|| {
            let listed: Vec<String> = candidates
                .iter()
                .map(|candidate| candidate.display().to_string())
                .collect();
            format!(
                "no credential file to probe ({}); log the CLI in first, otherwise a denial \
                 proves nothing",
                listed.join(", ")
            )
        })
}

pub fn redact(text: &str, home: Option<&str>) -> String {
    match home {
        Some(home) if !home.is_empty() => text.replace(home, "~"),
        _ => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    fn host(pairs: &[(&str, &str)]) -> HostEnvironment {
        HostEnvironment::from_variables(
            pairs
                .iter()
                .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        )
    }

    #[test]
    fn codex_probes_the_login_in_the_codex_home_coach_passes() {
        let coach_home = Path::new("/coach/codex-home");
        for platform in [Platform::MacOs, Platform::Windows] {
            let candidates = credential_candidates(
                CliAdapter::Codex,
                platform,
                &host(&[
                    ("HOME", "/Users/learner"),
                    ("USERPROFILE", r"C:\Users\Learner"),
                ]),
                coach_home,
            )
            .unwrap();
            assert_eq!(candidates, [coach_home.join("auth.json")]);
        }
    }

    #[test]
    fn claude_files_live_under_home_on_macos_and_the_profile_on_windows() {
        let both = host(&[
            ("HOME", "/Users/learner"),
            ("USERPROFILE", r"C:\Users\Learner"),
        ]);
        let codex_home = Path::new("/unused");
        for (platform, home) in [
            (Platform::MacOs, "/Users/learner"),
            (Platform::Windows, r"C:\Users\Learner"),
        ] {
            let home = Path::new(home);
            assert_eq!(
                credential_candidates(CliAdapter::Claude, platform, &both, codex_home).unwrap(),
                [
                    home.join(".claude").join(".credentials.json"),
                    home.join(".claude.json")
                ]
            );
        }
    }

    #[test]
    fn a_missing_or_empty_profile_variable_fails_instead_of_probing_the_root() {
        let windows_without_profile = host(&[("HOME", "/Users/learner"), ("USERPROFILE", "")]);
        let error = credential_candidates(
            CliAdapter::Claude,
            Platform::Windows,
            &windows_without_profile,
            Path::new("/unused"),
        )
        .unwrap_err();
        assert!(error.contains("USERPROFILE"), "{error}");
        assert!(
            credential_candidates(
                CliAdapter::Claude,
                Platform::MacOs,
                &host(&[]),
                Path::new("/unused")
            )
            .unwrap_err()
            .contains("HOME")
        );
    }

    #[test]
    fn the_probe_targets_the_first_existing_file_and_fails_when_none_exists() {
        let root = tempfile::tempdir().unwrap();
        let credentials = root.path().join(".credentials.json");
        let config = root.path().join(".claude.json");
        let candidates = [credentials.clone(), config.clone()];
        let error = existing_target(&candidates).unwrap_err();
        assert!(error.contains("log the CLI in first"), "{error}");
        std::fs::write(&config, "{}").unwrap();
        assert_eq!(existing_target(&candidates).unwrap(), config);
        std::fs::write(&credentials, "{}").unwrap();
        assert_eq!(existing_target(&candidates).unwrap(), credentials);
    }

    #[test]
    fn a_directory_is_not_a_credential_file() {
        let root = tempfile::tempdir().unwrap();
        assert!(existing_target(&[root.path().to_path_buf()]).is_err());
    }

    #[test]
    fn redaction_replaces_the_home_and_leaves_text_alone_without_one() {
        let text = r"C:\Users\Learner\.claude.json denied";
        assert_eq!(
            redact(text, Some(r"C:\Users\Learner")),
            r"~\.claude.json denied"
        );
        assert_eq!(redact(text, Some("")), text);
        assert_eq!(redact(text, None), text);
        assert_eq!(redact("/Users/learner/x", Some("/Users/learner")), "~/x");
    }
}
