mod capture;
#[cfg(unix)]
mod group;
#[cfg(windows)]
mod job;

use std::ffi::OsString;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Notify;

use crate::adapter::invocation::Invocation;
use capture::Capture;
#[cfg(unix)]
use group::OwnedTree;
#[cfg(windows)]
use job::OwnedTree;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub output_bytes: u64,
    pub deadline: Duration,
    pub grace: Duration,
}

#[derive(Clone, Debug)]
pub struct ProcessSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    // The complete child environment; nothing is inherited from Coach's own process.
    pub env: Vec<(OsString, OsString)>,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
    pub limits: Limits,
}

impl ProcessSpec {
    pub fn from_invocation(
        invocation: Invocation,
        stdout: PathBuf,
        stderr: PathBuf,
        limits: Limits,
    ) -> Self {
        Self {
            program: invocation.program,
            args: invocation.args,
            cwd: invocation.cwd,
            env: invocation.env,
            stdout,
            stderr,
            limits,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Exited(Option<i32>),
    TimedOut,
    Cancelled,
    OutputLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamSummary {
    pub bytes: u64,
    pub truncated: bool,
    pub closed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessReport {
    pub ending: Ending,
    pub stdout: StreamSummary,
    pub stderr: StreamSummary,
    pub elapsed: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("cannot create output file {path}: {source}")]
    Output { path: PathBuf, source: io::Error },
    #[error("cannot start {program}: {source}")]
    Spawn { program: PathBuf, source: io::Error },
    #[error("lost track of the child process: {0}")]
    Wait(io::Error),
}

const DRAIN_AFTER_EXIT: Duration = Duration::from_secs(2);

// Runs one child with null stdin, both streams drained to bounded files, and the whole process tree
// terminated on timeout, cancellation, output overflow or after the main process exits.
pub async fn run(
    spec: ProcessSpec,
    cancel: impl Future<Output = ()>,
) -> Result<ProcessReport, ProcessError> {
    let started = Instant::now();
    let stdout_file = create(&spec.stdout).await?;
    let stderr_file = create(&spec.stderr).await?;
    let mut tree = OwnedTree::spawn(&spec).map_err(|source| ProcessError::Spawn {
        program: spec.program.clone(),
        source,
    })?;
    let overflow = Arc::new(Notify::new());
    let limit = spec.limits.output_bytes;
    let stdout = Capture::start(
        tree.take_stdout(),
        stdout_file,
        limit,
        Arc::clone(&overflow),
    );
    let stderr = Capture::start(
        tree.take_stderr(),
        stderr_file,
        limit,
        Arc::clone(&overflow),
    );
    tokio::pin!(cancel);
    let ending = tokio::select! {
        biased;
        () = overflow.notified() => Ending::OutputLimitExceeded,
        () = &mut cancel => Ending::Cancelled,
        () = tokio::time::sleep(spec.limits.deadline) => Ending::TimedOut,
        status = tree.wait_main() => Ending::Exited(status.map_err(ProcessError::Wait)?.code()),
    };
    if !matches!(ending, Ending::Exited(_)) {
        tree.terminate(spec.limits.grace).await;
    }
    tree.kill_remaining();
    let stdout = stdout.finish(DRAIN_AFTER_EXIT).await;
    let stderr = stderr.finish(DRAIN_AFTER_EXIT).await;
    Ok(ProcessReport {
        ending,
        stdout,
        stderr,
        elapsed: started.elapsed(),
    })
}

async fn create(path: &std::path::Path) -> Result<tokio::fs::File, ProcessError> {
    tokio::fs::File::create(path)
        .await
        .map_err(|source| ProcessError::Output {
            path: path.to_path_buf(),
            source,
        })
}
