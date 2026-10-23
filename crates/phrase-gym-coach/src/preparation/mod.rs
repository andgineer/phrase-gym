pub mod attempt;
mod prompt;
pub mod readiness;
mod reference;
pub mod result;
mod schema;
pub mod selection;
pub mod workspace;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use phrase_gym_contracts::inbox::{JobClaim, LessonOutput};

use crate::platform::paths::CoachPaths;
use crate::process;
use crate::transfer::client::ServerClient;

pub fn prepare(
    client: &ServerClient,
    paths: &CoachPaths,
    claim: &JobClaim,
    tool: &readiness::ClaudeTool,
    shutdown: &Arc<AtomicBool>,
) -> Result<LessonOutput, String> {
    prepare_selected(
        client,
        paths,
        claim,
        &selection::SelectedTool::claude(tool),
        shutdown,
    )
}

pub fn prepare_selected(
    client: &ServerClient,
    paths: &CoachPaths,
    claim: &JobClaim,
    tool: &selection::SelectedTool<'_>,
    shutdown: &Arc<AtomicBool>,
) -> Result<LessonOutput, String> {
    client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
    let previous = tool.attempts(paths).join(claim.fence.token.to_string());
    if previous.exists() {
        let bytes = crate::platform::files::read_selected(&previous.join("validated.json"), 100_000)
            .map_err(|_| "The previous selected CLI attempt was interrupted before validation. Reconcile the accepted lesson and explicitly retry preparation; it will not be started twice")?;
        let output: LessonOutput = serde_json::from_slice(&bytes).map_err(|_| "Saved validated output is unreadable; preserve the private attempt before retrying")?;
        if output.fence != claim.fence {
            return Err("Saved output belongs to a different fenced attempt".into());
        }
        return Ok(output);
    }
    attempt::validate_input(claim)?;
    let staged = attempt::stage_selected(paths, claim, tool)?;
    let spec = staged.spec;
    let started = Instant::now();
    let cancel = Arc::new(AtomicBool::new(false));
    let stop = Arc::clone(&cancel);
    let (send, receive) = mpsc::sync_channel(1);
    let child = std::thread::spawn(move || {
        let result = readiness::execute(spec, &stop);
        let _ = send.send(result);
    });
    let mut renewed = Instant::now();
    let mut lease_failure = None;
    let report = loop {
        match receive.recv_timeout(Duration::from_millis(100)) {
            Ok(report) => break report,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break Err("Preparation supervisor stopped".into());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if shutdown.load(Ordering::SeqCst) {
            cancel.store(true, Ordering::SeqCst);
        }
        if renewed.elapsed() >= Duration::from_secs(30) && lease_failure.is_none() {
            match client.post::<_, String>("/api/v1/preparation/renew", &claim.fence) {
                Ok(_) => renewed = Instant::now(),
                Err(problem) => {
                    lease_failure = Some(problem);
                    cancel.store(true, Ordering::SeqCst);
                }
            }
        }
    };
    child.join().map_err(|_| "Preparation supervisor failed")?;
    if let Some(problem) = lease_failure {
        return Err(format!(
            "Preparation stopped because its lease could not be renewed: {problem}"
        ));
    }
    if shutdown.load(Ordering::SeqCst) {
        return Err("Preparation cancelled; accepted input remains on Server".into());
    }
    let mut report = report?;
    let usage = match tool.adapter {
        crate::adapter::CliAdapter::Claude => crate::adapter::claude_output::usage(
            &crate::platform::files::read_selected(&staged.directory.join("stdout.log"), 64 * 1024)
                .unwrap_or_default(),
        ),
        crate::adapter::CliAdapter::Codex => serde_json::Value::Null,
    };
    crate::platform::files::save(
        &staged.directory.join("execution.json"),
        &serde_json::json!({
            "adapter": tool.identity(), "elapsed_ms": started.elapsed().as_millis(),
            "ending": format!("{:?}", report.ending), "usage": usage,
            "stdout_bytes": report.stdout.bytes, "stderr_bytes": report.stderr.bytes
        }),
    )?;
    if report.stdout.truncated || report.stderr.truncated {
        report.ending = process::Ending::OutputLimitExceeded;
    }
    if report.ending != process::Ending::Exited(Some(0)) {
        return Err(result::failure(&staged.directory, report.ending));
    }
    if !report.stdout.closed || !report.stderr.closed {
        return Err(
            "selected CLI output streams did not close; no result can be validated or published"
                .into(),
        );
    }
    let mut output =
        result::read_selected(&staged.directory, claim, tool.adapter, &tool.identity())?;
    reference::complete(client, paths, claim, &mut output, shutdown)?;
    // A successful process cannot authorize publication after a lost or replaced lease.
    client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
    crate::platform::files::save(&previous.join("validated.json"), &output)?;
    Ok(output)
}
