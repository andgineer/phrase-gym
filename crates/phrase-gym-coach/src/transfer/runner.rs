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
use super::material::Material;
use super::pending::{self, PendingSend};
use crate::supplied::SuppliedLesson;

pub fn create(server: &str, json: &str, audio: &Path) -> Result<PendingSend, String> {
    let supplied = SuppliedLesson::parse(json)?;
    create_material(server, Material::Supplied(Box::new(supplied)), audio)
}

pub fn create_material(
    server: &str,
    material: Material,
    audio: &Path,
) -> Result<PendingSend, String> {
    let bytes = crate::platform::files::read_selected(audio, pending::MAX_AUDIO)?;
    if bytes.is_empty() {
        return Err("Select the supplied original reference clip".into());
    }
    Ok(PendingSend {
        server: server.to_owned(),
        identity: None,
        failure: None,
        audio_sha256: pending::audio_hash(&bytes),
        output: None,
        submission_id: Uuid::new_v4(),
        claim_id: Uuid::new_v4(),
        upload: MediaUpload {
            metadata: material.metadata(),
            mime_type: material.mime(),
            audio_base64: STANDARD.encode(bytes),
        },
        material,
        submission: None,
    })
}

pub fn deliver(
    client: &ServerClient,
    path: &Path,
    pending: &mut PendingSend,
    mut status: impl FnMut(&str),
) -> Result<PublicationReceipt, String> {
    deliver_with(client, path, pending, &mut status, crate::supplied::output)
}

pub fn deliver_with(
    client: &ServerClient,
    path: &Path,
    pending: &mut PendingSend,
    mut status: impl FnMut(&str),
    prepare: impl FnOnce(&JobClaim) -> Result<phrase_gym_contracts::inbox::LessonOutput, String>,
) -> Result<PublicationReceipt, String> {
    let result = attempt(client, path, pending, &mut status, prepare);
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
    pending.output = None;
    pending::save(path, pending)
}

fn attempt(
    client: &ServerClient,
    path: &Path,
    pending: &mut PendingSend,
    status: &mut impl FnMut(&str),
    prepare: impl FnOnce(&JobClaim) -> Result<phrase_gym_contracts::inbox::LessonOutput, String>,
) -> Result<PublicationReceipt, String> {
    pending.verify(&client.server, client.identity)?;
    pending.identity = Some(client.identity);
    pending::save(path, pending)?;
    if let Some(reason) = &pending.failure {
        return Err(format!(
            "Accepted lesson needs attention: {reason}. Use Retry preparation after fixing it"
        ));
    }
    if pending.submission.is_none() {
        status("Connected; transferring supplied reference audio");
        let media: MediaAsset = client.post("/api/v1/media", &pending.upload)?;
        if media.sha256 != pending.audio_sha256 {
            return Err("Server audio acknowledgement differs from the retained original hash; nothing was submitted".into());
        }
        pending.submission = Some(pending.material.submission(pending.submission_id, &media)?);
        pending::save(path, pending)?;
    }
    let input = pending
        .submission
        .as_ref()
        .ok_or("Pending Send has no input")?;
    let receipt: LessonReceipt = client.post("/api/v1/inbox/lessons", input)?;
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
        client.post("/api/v1/preparation/receipt", &claim.fence)?;
    if let Some(publication) = committed {
        return finish(path, publication);
    }
    status("Preparing supplied lesson under the Server lease");
    client.post::<_, String>("/api/v1/preparation/renew", &claim.fence)?;
    if pending.output.is_none() {
        pending.output = Some(match prepare(&claim) {
            Ok(output) => output,
            Err(reason) => {
                pending.failure = Some(reason.clone());
                pending::save(path, pending)?;
                let failure = phrase_gym_contracts::inbox::JobFailure {
                    fence: claim.fence,
                    reason: reason.clone(),
                };
                client.post::<_, ()>("/api/v1/preparation/fail", &failure)?;
                return Err(reason);
            }
        });
        pending::save(path, pending)?;
    }
    if let Err(problem) = client.post::<_, ()>("/api/v1/preparation/stage", &pending.output) {
        if problem.starts_with("Server 422") {
            let reason = format!("Prepared lesson failed validation: {problem}");
            pending.failure = Some(reason.clone());
            pending::save(path, pending)?;
            client.post::<_, ()>(
                "/api/v1/preparation/fail",
                &phrase_gym_contracts::inbox::JobFailure {
                    fence: claim.fence,
                    reason: reason.clone(),
                },
            )?;
            return Err(reason);
        }
        return Err(problem);
    }
    status("Supplied lesson staged; reconciling atomic publication");
    let publication = match client.post("/api/v1/preparation/publish", &claim.fence) {
        Ok(publication) => publication,
        Err(problem) => client
            .post::<_, Option<PublicationReceipt>>("/api/v1/preparation/receipt", &claim.fence)?
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
