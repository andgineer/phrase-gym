use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::adapter::{
    CliAdapter, claude,
    environment::{HostEnvironment, child_environment},
};
use crate::platform::tools::{self, Discovery};
use crate::process::{self, Limits, ProcessReport, ProcessSpec};

// The oldest build whose isolation and structured output were measured; the native installer
// only moves forward, and every newer build must still offer each isolation flag.
pub const MINIMUM_VERSION: [u32; 3] = [2, 1, 309];
pub const MODEL: &str = "sonnet";

pub struct ClaudeTool {
    pub executable: PathBuf,
    pub host: HostEnvironment,
    pub version: String,
}

pub fn execute(spec: ProcessSpec, cancel: &AtomicBool) -> Result<ProcessReport, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Cannot initialize preparation supervision")?;
    runtime
        .block_on(process::run(spec, async {
            while !cancel.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }))
        .map_err(|_| {
            "Cannot start or supervise the selected native tool; check its executable".into()
        })
}

pub fn supported_version(output: &str) -> Result<String, String> {
    let version = output
        .trim()
        .strip_suffix(" (Claude Code)")
        .ok_or("Claude reported an unreadable version; reinstall the native Claude CLI")?;
    let parts: Vec<u32> = version
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| "Claude reported an unreadable version; reinstall the native Claude CLI")?;
    if parts.len() != 3 || parts.as_slice() < MINIMUM_VERSION.as_slice() {
        let [major, minor, patch] = MINIMUM_VERSION;
        return Err(format!(
            "Claude {version} is older than {major}.{minor}.{patch}, the oldest build whose isolation was verified; update Claude Code"
        ));
    }
    Ok(version.to_owned())
}

impl ClaudeTool {
    pub fn check(directory: &Path) -> Result<Self, String> {
        let host = HostEnvironment::current();
        let Discovery::Native(executable) = tools::discover(CliAdapter::Claude, &host) else {
            return Err(
                "Install the native Claude CLI and sign in with your Claude subscription".into(),
            );
        };
        let mut tool = Self {
            executable,
            host,
            version: String::new(),
        };
        tool.version = supported_version(&tool.probe(directory, &["--version"], "version")?)?;
        let missing = claude::missing_flags(&tool.probe(directory, &["--help"], "help")?);
        if !missing.is_empty() {
            return Err(format!(
                "Claude {} no longer offers {}; Coach's preparation isolation must be re-verified before Claude can prepare cards",
                tool.version,
                missing.join(", ")
            ));
        }
        let auth = tool.probe(directory, &["auth", "status"], "auth")?;
        let auth: serde_json::Value = serde_json::from_str(&auth).map_err(
            |_| "Claude login status is unreadable; run claude auth status outside Coach",
        )?;
        if auth["loggedIn"] != true || auth["authMethod"] != "claude.ai" {
            return Err("Claude subscription login required; run claude auth login outside Coach. API keys are not used".into());
        }
        Ok(tool)
    }

    fn probe(&self, directory: &Path, arguments: &[&str], name: &str) -> Result<String, String> {
        let stdout = directory.join(format!("{name}.stdout"));
        let stderr = directory.join(format!("{name}.stderr"));
        let mut env = child_environment(
            &self.host,
            self.executable
                .parent()
                .ok_or("Claude path has no parent")?,
        );
        env.push((
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".into(),
            "1".into(),
        ));
        let report = execute(
            ProcessSpec {
                program: self.executable.clone(),
                args: arguments.iter().map(Into::into).collect(),
                cwd: directory.into(),
                env,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
                limits: Limits {
                    output_bytes: 64 * 1024,
                    deadline: Duration::from_secs(15),
                    grace: Duration::from_secs(1),
                },
            },
            &AtomicBool::new(false),
        )?;
        let bytes =
            std::fs::read_to_string(&stdout).map_err(|_| "Cannot read Claude readiness result")?;
        // Authentication output can include account identifiers; retain no readiness transcript.
        let _ = std::fs::remove_file(stdout);
        let _ = std::fs::remove_file(stderr);
        if report.ending != process::Ending::Exited(Some(0)) || report.stdout.truncated {
            return Err(
                "Claude is unavailable or logged out; run claude auth status outside Coach".into(),
            );
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_verified_build_and_later_updates_but_not_older_or_foreign_versions() {
        assert_eq!(
            supported_version("2.1.309 (Claude Code)\n").unwrap(),
            "2.1.309"
        );
        for newer in ["2.1.310", "2.2.0", "3.0.0", "2.1.1000"] {
            assert_eq!(
                supported_version(&format!("{newer} (Claude Code)")).unwrap(),
                newer
            );
        }
        for older in ["2.1.302", "2.0.999", "1.9.400"] {
            let problem = supported_version(&format!("{older} (Claude Code)")).unwrap_err();
            assert!(problem.contains("older than 2.1.309"), "{problem}");
        }
        for unreadable in ["2.1.309", "2.1 (Claude Code)", "two (Claude Code)", ""] {
            assert!(supported_version(unreadable).is_err(), "{unreadable:?}");
        }
    }
}
