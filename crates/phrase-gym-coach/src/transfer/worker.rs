use std::path::Path;

use phrase_gym_contracts::inbox::{
    ClaimRequest, JobClaim, JobFailure, LessonOutput, PublicationReceipt,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::client::ServerClient;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    identity: Uuid,
    claim_id: Uuid,
    output: Option<LessonOutput>,
}

pub fn tick(
    client: &ServerClient,
    path: &Path,
    prepare: impl FnOnce(&JobClaim) -> Result<LessonOutput, String>,
) -> Result<Option<PublicationReceipt>, String> {
    let result = attempt(client, path, prepare);
    if result.as_ref().is_err_and(|problem| {
        problem.starts_with("Server 409")
            && (problem.contains("Unknown worker delivery")
                || problem.contains("Worker lease expired"))
    }) {
        std::fs::remove_file(path)
            .map_err(|_| "Cannot clear expired worker checkpoint; preserve it before recovery")?;
        return Err("Expired preparation reconciled without a publication; reconnecting to accepted Server work".into());
    }
    result
}

fn attempt(
    client: &ServerClient,
    path: &Path,
    prepare: impl FnOnce(&JobClaim) -> Result<LessonOutput, String>,
) -> Result<Option<PublicationReceipt>, String> {
    let mut checkpoint = if path.exists() {
        serde_json::from_slice::<Checkpoint>(&crate::platform::files::read_selected(path, 100_000)?)
            .map_err(|_| "Worker recovery state is unreadable; preserve it before repair")?
    } else {
        Checkpoint {
            identity: client.identity,
            claim_id: Uuid::new_v4(),
            output: None,
        }
    };
    if checkpoint.identity != client.identity {
        return Err("Unfinished preparation belongs to the original Server connection; restore it before switching accounts".into());
    }
    crate::platform::files::save(path, &checkpoint)?;
    let claim: Option<JobClaim> = client.post(
        "/api/v1/preparation/claims",
        &ClaimRequest {
            claim_id: checkpoint.claim_id,
        },
    )?;
    let Some(claim) = claim else {
        std::fs::remove_file(path).map_err(|_| "Cannot clear idle worker checkpoint")?;
        return Ok(None);
    };
    let receipt: Option<PublicationReceipt> =
        client.post("/api/v1/preparation/receipt", &claim.fence)?;
    if let Some(receipt) = receipt {
        return finish(path, receipt).map(Some);
    }
    client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
    if checkpoint.output.is_none() {
        match prepare(&claim) {
            Ok(output) => {
                checkpoint.output = Some(output);
                crate::platform::files::save(path, &checkpoint)?;
            }
            Err(reason) => {
                // Failure acknowledgement is fenced. A lost response retains the checkpoint so a
                // reconnect cannot silently spend another start or publish stale output.
                client.post::<_, ()>(
                    "/api/v1/preparation/fail",
                    &JobFailure {
                        fence: claim.fence,
                        reason: reason.clone(),
                    },
                )?;
                std::fs::remove_file(path).map_err(|_| "Cannot clear failed worker checkpoint")?;
                return Err(reason);
            }
        }
    }
    if let Err(problem) = client.post::<_, ()>("/api/v1/preparation/stage", &checkpoint.output) {
        if problem.starts_with("Server 422") {
            let reason = format!("Prepared lesson failed validation: {problem}");
            client.post::<_, ()>(
                "/api/v1/preparation/fail",
                &JobFailure {
                    fence: claim.fence,
                    reason: reason.clone(),
                },
            )?;
            std::fs::remove_file(path).map_err(|_| "Cannot clear rejected output checkpoint")?;
            return Err(reason);
        }
        return Err(problem);
    }
    let receipt = match client.post("/api/v1/preparation/publish", &claim.fence) {
        Ok(receipt) => receipt,
        Err(problem) => client
            .post::<_, Option<PublicationReceipt>>("/api/v1/preparation/receipt", &claim.fence)?
            .ok_or(problem)?,
    };
    finish(path, receipt).map(Some)
}

fn finish(path: &Path, receipt: PublicationReceipt) -> Result<PublicationReceipt, String> {
    std::fs::remove_file(path)
        .map_err(|_| "Published; preserve worker recovery state and reconnect to reconcile")?;
    Ok(receipt)
}
