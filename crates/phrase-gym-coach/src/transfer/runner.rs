use std::fs;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use phrase_gym_contracts::inbox::{
    ClaimRequest, JobClaim, LessonReceipt, LessonStatus, PublicationReceipt,
};
use phrase_gym_contracts::media::{MediaAsset, MediaUpload};
use uuid::Uuid;

use super::client::ServerClient;
use super::pending::{self, PendingSend};
use crate::supplied::{self, SuppliedLesson};

pub fn create(server: &str, json: &str, audio: &Path) -> Result<PendingSend, String> {
    let supplied = SuppliedLesson::parse(json)?;
    let bytes = pending::read_selected(audio, pending::MAX_AUDIO)?;
    if bytes.is_empty() {
        return Err("Select the supplied original reference clip".into());
    }
    Ok(PendingSend {
        server: server.to_owned(),
        submission_id: Uuid::new_v4(),
        claim_id: Uuid::new_v4(),
        upload: MediaUpload {
            metadata: supplied.metadata.clone(),
            mime_type: supplied.mime_type,
            audio_base64: STANDARD.encode(bytes),
        },
        supplied,
        submission: None,
    })
}

pub fn deliver(
    client: &ServerClient,
    path: &Path,
    pending: &mut PendingSend,
    mut status: impl FnMut(&str),
) -> Result<PublicationReceipt, String> {
    let result = attempt(client, path, pending, &mut status);
    if result
        .as_ref()
        .is_err_and(|problem| problem.starts_with("Server 409"))
    {
        reset_claim(path, pending)?;
    }
    result
}

fn reset_claim(path: &Path, pending: &mut PendingSend) -> Result<(), String> {
    pending.claim_id = Uuid::new_v4();
    pending::save(path, pending)
}

fn attempt(
    client: &ServerClient,
    path: &Path,
    pending: &mut PendingSend,
    status: &mut impl FnMut(&str),
) -> Result<PublicationReceipt, String> {
    if pending.server != client.server {
        return Err("Connect to the pending Send's original Server before Retry".into());
    }
    pending::save(path, pending)?;
    if pending.submission.is_none() {
        status("Connected; transferring supplied reference audio");
        let media: MediaAsset = client.post("/api/v1/media", &pending.upload, false)?;
        pending.submission = Some(pending.supplied.submission(pending.submission_id, &media)?);
        pending::save(path, pending)?;
    }
    let input = pending
        .submission
        .as_ref()
        .ok_or("Pending Send has no input")?;
    let receipt: LessonReceipt = client.post("/api/v1/inbox/lessons", input, false)?;
    status("Accepted durably; waiting for supplied-lesson preparation");
    let current: LessonStatus =
        client.get(&format!("/api/v1/inbox/lessons/{}", receipt.generation_id))?;
    if let Some(publication) = current.publication {
        return finish(path, publication);
    }
    let claim: Option<JobClaim> = client.post(
        "/api/v1/preparation/claims",
        &ClaimRequest {
            claim_id: pending.claim_id,
        },
        true,
    )?;
    let Some(claim) = claim else {
        reset_claim(path, pending)?;
        return Err("Accepted and waiting; another worker may hold the lease. Retry later".into());
    };
    if claim.receipt.generation_id != receipt.generation_id {
        reset_claim(path, pending)?;
        return Err("Accepted and waiting; an earlier inbox item needs its worker. No supplied output was published".into());
    }
    let committed: Option<PublicationReceipt> =
        client.post("/api/v1/preparation/receipt", &claim.fence, true)?;
    if let Some(publication) = committed {
        return finish(path, publication);
    }
    status("Preparing supplied lesson under the Server lease");
    client.post::<_, String>("/api/v1/preparation/renew", &claim.fence, true)?;
    let output = supplied::output(&claim)?;
    client.post::<_, ()>("/api/v1/preparation/stage", &output, true)?;
    status("Supplied lesson staged; reconciling atomic publication");
    let publication = match client.post("/api/v1/preparation/publish", &claim.fence, true) {
        Ok(publication) => publication,
        Err(problem) => client
            .post::<_, Option<PublicationReceipt>>(
                "/api/v1/preparation/receipt",
                &claim.fence,
                true,
            )?
            .ok_or(problem)?,
    };
    finish(path, publication)
}

fn finish(path: &Path, publication: PublicationReceipt) -> Result<PublicationReceipt, String> {
    fs::remove_file(path).map_err(
        |_| "Published; cannot clear local pending Send. Retry safely reconciles its receipt",
    )?;
    Ok(publication)
}
