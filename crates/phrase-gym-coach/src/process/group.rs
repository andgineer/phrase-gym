use std::io;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use rustix::process::{Pid, Signal, kill_process_group};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};

use super::ProcessSpec;

// The child leads its own process group so every descendant that stays in it can be signalled at
// once. Group signalling is cleanup, not a security boundary: a descendant may leave the group.
pub(super) struct OwnedTree {
    child: Child,
    group: Option<Pid>,
}

impl OwnedTree {
    pub(super) fn spawn(spec: &ProcessSpec) -> io::Result<Self> {
        let child = Command::new(&spec.program)
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .env_clear()
            .envs(spec.env.iter().map(|(key, value)| (key, value)))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true)
            .spawn()?;
        let group = child
            .id()
            .and_then(|id| i32::try_from(id).ok())
            .and_then(Pid::from_raw);
        Ok(Self { child, group })
    }

    pub(super) fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    pub(super) fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    pub(super) async fn wait_main(&mut self) -> io::Result<ExitStatus> {
        self.child.wait().await
    }

    pub(super) async fn terminate(&mut self, grace: Duration) {
        self.signal(Signal::TERM);
        if tokio::time::timeout(grace, self.child.wait())
            .await
            .is_err()
        {
            self.signal(Signal::KILL);
            let _ = self.child.wait().await;
        }
    }

    // Once the group has emptied, its id may be reused by an unrelated group, so it is signalled
    // only once more and then forgotten.
    pub(super) fn kill_remaining(&mut self) {
        self.signal(Signal::KILL);
        self.group = None;
    }

    fn signal(&self, signal: Signal) {
        if let Some(group) = self.group {
            let _ = kill_process_group(group, signal);
        }
    }
}

impl Drop for OwnedTree {
    fn drop(&mut self) {
        self.signal(Signal::KILL);
    }
}
