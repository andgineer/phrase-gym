use std::path::Path;

use phrase_gym_contracts::inbox::{LessonReceipt, LessonStatus};
use uuid::Uuid;

use super::{client::ServerClient, pending};

pub fn preparation(client: &ServerClient, path: &Path) -> Result<(), String> {
    let mut retained = pending::load(path)?.ok_or("No retained lesson is available to retry")?;
    retained.verify(&client.server, client.identity)?;
    if retained.identity != Some(client.identity) {
        return Err(
            "Restore the original pending input connection before retrying preparation".into(),
        );
    }
    let input = retained
        .submission
        .as_ref()
        .ok_or("Delivery is not yet accepted; restore the connection first")?;
    let receipt: Option<LessonReceipt> = client.get(&format!(
        "/api/v1/inbox/submissions/{}",
        input.submission_id
    ))?;
    let receipt =
        receipt.ok_or("Server has not confirmed this submission; restore delivery first")?;
    client.post::<_, LessonStatus>(
        &format!("/api/v1/inbox/lessons/{}/retry", receipt.generation_id),
        &(),
    )?;
    retained.claim_id = Uuid::new_v4();
    retained.output = None;
    retained.failure = None;
    pending::save(path, &retained)
}
