use std::future::pending;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::CliAdapter;
use super::environment::{HostEnvironment, child_environment};
use crate::platform::paths::CoachPaths;
use crate::platform::tools::{self, Discovery};
use crate::process::{Ending, Limits, ProcessSpec};

pub const VERIFIED_VERSION: &str = "0.159.2";
pub const MODEL: &str = "gpt-5.6-luna";

pub struct CodexReady {
    pub executable: PathBuf,
    pub home: PathBuf,
    pub host: HostEnvironment,
}

// This is setup readiness only; it neither starts a model nor certifies own-home isolation.
pub fn check() -> Result<CodexReady, String> {
    let paths = CoachPaths::for_current_user().map_err(|e| e.to_string())?;
    let host = HostEnvironment::current();
    let home = checked_home(&paths, &host)?;
    let Discovery::Native(executable) = tools::discover(CliAdapter::Codex, &host) else {
        return Err("Install the native Codex CLI; script installs are unsupported".into());
    };
    let scratch = paths
        .cache
        .join(format!("codex-readiness-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&paths.cache).map_err(|_| "Cannot create Coach cache")?;
    std::fs::create_dir(&scratch).map_err(|_| "Cannot create private readiness directory")?;
    let cleanup = Scratch(scratch);
    #[cfg(unix)]
    {
        std::fs::set_permissions(&cleanup.0, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| "Cannot restrict readiness directory")?;
    }
    check_with(&executable, &home, &host, &cleanup.0, probe)?;
    Ok(CodexReady {
        executable,
        home,
        host,
    })
}

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn checked_home(paths: &CoachPaths, host: &HostEnvironment) -> Result<PathBuf, String> {
    let home = paths.codex_home();
    for path in [&paths.state, &home] {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| {
            "Coach Codex home is missing; follow the attended Codex setup in docs/coach-desktop.md"
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("Coach state and Codex home must be real directories, not aliases".into());
        }
    }
    let home = home
        .canonicalize()
        .map_err(|_| "Cannot resolve Coach Codex home")?;
    for variable in ["HOME", "USERPROFILE"] {
        if let Some(profile) = host.get(variable)
            && Path::new(profile)
                .join(".codex")
                .canonicalize()
                .is_ok_and(|p| p == home)
        {
            return Err("Coach cannot use the operator's Codex home".into());
        }
    }
    for name in ["AGENTS.md", "AGENTS.override.md"] {
        if std::fs::symlink_metadata(home.join(name)).is_ok() {
            return Err(
                "Remove user instructions from Coach's dedicated Codex home before measurement"
                    .into(),
            );
        }
    }
    let skills = home.join("skills");
    if std::fs::symlink_metadata(&skills).is_ok() {
        if std::fs::symlink_metadata(&skills).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("Coach Codex skills must not be a symlink".into());
        }
        for entry in std::fs::read_dir(&skills).map_err(|_| "Cannot inspect Coach Codex skills")? {
            let entry = entry.map_err(|_| "Cannot inspect Coach Codex skills")?;
            if entry.file_name() != ".system"
                || entry
                    .file_type()
                    .map_err(|_| "Cannot inspect skill")?
                    .is_symlink()
            {
                return Err(
                    "Remove custom skills from Coach's dedicated Codex home before measurement"
                        .into(),
                );
            }
        }
    }
    Ok(home)
}

fn check_with(
    executable: &Path,
    home: &Path,
    host: &HostEnvironment,
    scratch: &Path,
    mut run: impl FnMut(ProcessSpec) -> Result<String, String>,
) -> Result<(), String> {
    let mut env = child_environment(host, executable.parent().ok_or("Invalid Codex executable")?);
    env.push(("CODEX_HOME".into(), home.as_os_str().to_owned()));
    let mut inspect = |args: &[&str]| {
        run(ProcessSpec {
            program: executable.into(),
            args: args.iter().map(Into::into).collect(),
            cwd: scratch.into(),
            env: env.clone(),
            stdout: scratch.join("stdout"),
            stderr: scratch.join("stderr"),
            limits: Limits {
                output_bytes: 16_384,
                deadline: Duration::from_secs(15),
                grace: Duration::from_secs(1),
            },
        })
    };
    if inspect(&["--version"])?.trim() != format!("codex-cli {VERIFIED_VERSION}") {
        return Err(format!(
            "Codex invocation was measured only on {VERIFIED_VERSION}; repeat isolation acceptance before using another version"
        ));
    }
    // Status inspection deliberately omits forced_login_method: an API login must be rejected,
    // never logged out or mutated by this no-spend check.
    if inspect(&["--ignore-user-config", "login", "status"])?.trim() != "Logged in using ChatGPT" {
        return Err("Coach's own ChatGPT subscription login is required; perform the attended Codex setup in docs/coach-desktop.md. No login was started".into());
    }
    Ok(())
}

fn probe(spec: ProcessSpec) -> Result<String, String> {
    let stdout = spec.stdout.clone();
    let stderr = spec.stderr.clone();
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Cannot initialize readiness supervision")?
        .block_on(crate::process::run(spec, pending()));
    let output = std::fs::read_to_string(&stdout);
    let errors = std::fs::read_to_string(&stderr);
    // Codex login status currently uses stderr; retain no account or authentication transcript.
    let clean = [stdout, stderr]
        .iter()
        .all(|p| std::fs::remove_file(p).is_ok() || !p.exists());
    if !clean {
        return Err("Cannot remove private Codex readiness transcript".into());
    }
    let report = result.map_err(|_| "Cannot supervise Codex readiness check")?;
    if report.ending != Ending::Exited(Some(0))
        || report.stdout.truncated
        || report.stderr.truncated
        || !report.stdout.closed
        || !report.stderr.closed
    {
        return Err(
            "Codex readiness check failed; inspect version/login status manually in Coach's home"
                .into(),
        );
    }
    Ok(format!(
        "{}{}",
        output.map_err(|_| "Cannot read readiness result")?,
        errors.map_err(|_| "Cannot read readiness result")?
    ))
}

#[cfg(test)]
#[path = "codex_readiness_tests.rs"]
mod tests;
