use std::io;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use process_wrap::tokio::{ChildWrapper, CommandWrap, CreationFlags, JobObject, KillOnDrop};
use tokio::process::{ChildStderr, ChildStdout};
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use super::ProcessSpec;

// process-wrap creates the child suspended, assigns it to a kill-on-close Job Object and only then
// resumes it, so no descendant can start outside the job.
pub(super) struct OwnedTree {
    child: Box<dyn ChildWrapper>,
}

impl OwnedTree {
    pub(super) fn spawn(spec: &ProcessSpec) -> io::Result<Self> {
        let mut command = CommandWrap::with_new(&spec.program, |command| {
            command
                .args(&spec.args)
                .current_dir(&spec.cwd)
                .env_clear()
                .envs(spec.env.iter().map(|(key, value)| (key, value)))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        });
        // A GUI-subsystem parent would otherwise open a console window for every CLI child.
        command
            .wrap(CreationFlags(CREATE_NO_WINDOW))
            .wrap(JobObject)
            .wrap(KillOnDrop);
        Ok(Self {
            child: command.spawn()?,
        })
    }

    pub(super) fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout().take()
    }

    pub(super) fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr().take()
    }

    // The job-level wait also blocks on lingering descendants, which Coach kills instead.
    pub(super) async fn wait_main(&mut self) -> io::Result<ExitStatus> {
        self.child.inner_mut().wait().await
    }

    // Windowless console children have no graceful stop signal, so termination is immediate.
    pub(super) async fn terminate(&mut self, grace: Duration) {
        let _ = self.child.start_kill();
        let _ = tokio::time::timeout(grace, self.child.inner_mut().wait()).await;
    }

    pub(super) fn kill_remaining(&mut self) {
        let _ = self.child.start_kill();
    }
}
