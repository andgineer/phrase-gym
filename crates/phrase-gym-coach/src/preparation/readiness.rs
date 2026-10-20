use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::adapter::{
    CliAdapter,
    environment::{HostEnvironment, child_environment},
};
use crate::platform::tools::{self, Discovery};
use crate::process::{self, Limits, ProcessReport, ProcessSpec};

pub const VERIFIED_VERSION: &str = "2.1.302";
pub const MODEL: &str = "haiku";

pub struct ClaudeTool {
    pub executable: PathBuf,
    pub host: HostEnvironment,
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

impl ClaudeTool {
    pub fn check(directory: &Path) -> Result<Self, String> {
        let host = HostEnvironment::current();
        let Discovery::Native(mut executable) = tools::discover(CliAdapter::Claude, &host) else {
            return Err(
                "Install the native Claude CLI and sign in with your Claude subscription".into(),
            );
        };
        // The native install retains versioned binaries; use the measured build when updates move
        // the public symlink. Any other install still has to pass the exact version check below.
        let pinned = executable.with_file_name(VERIFIED_VERSION);
        if pinned.is_file() {
            executable = pinned
                .canonicalize()
                .map_err(|_| "Cannot resolve pinned Claude")?;
        }
        let tool = Self { executable, host };
        let version = tool.probe(directory, &["--version"], "version")?;
        if version.trim() != format!("{VERIFIED_VERSION} (Claude Code)") {
            return Err(format!(
                "Claude isolation is verified only for {VERIFIED_VERSION}; install that version or repeat the isolation acceptance before enabling preparation"
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
                    output_bytes: 16_384,
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
        if report.ending != process::Ending::Exited(Some(0)) {
            return Err(
                "Claude is unavailable or logged out; run claude auth status outside Coach".into(),
            );
        }
        Ok(bytes)
    }
}
