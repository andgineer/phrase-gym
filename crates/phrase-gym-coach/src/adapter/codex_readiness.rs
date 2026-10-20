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

pub const VERIFIED_VERSION: &str = "0.163.0";
pub const MODEL: &str = "gpt-5.6-luna";

pub struct CodexReady {
    pub executable: PathBuf,
    pub home: PathBuf,
    pub host: HostEnvironment,
}

// Readiness inspects the selected executable and subscription login without starting a model.
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

fn checked_home(_paths: &CoachPaths, host: &HostEnvironment) -> Result<PathBuf, String> {
    let profile = host
        .get("HOME")
        .or_else(|| host.get("USERPROFILE"))
        .ok_or("The operating system user home is unavailable")?;
    Path::new(profile).join(".codex").canonicalize()
        .map_err(|_| "Sign in to Codex with your ChatGPT subscription outside Coach; normal Codex home is missing".into())
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
    if inspect(&["login", "status"])?.trim() != "Logged in using ChatGPT" {
        return Err("ChatGPT subscription login in your normal Codex home is required; run codex login yourself outside Coach. No login was started".into());
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
            "Codex readiness check failed; inspect version/login status manually in your normal Codex home"
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
