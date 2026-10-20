use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use phrase_gym_coach::preparation;

use super::generated_server::{paths, raw, tool};
use super::*;

#[test]
fn replacement_attempt_can_stage_changed_output_without_reusing_or_duplicating_publication() {
    let (directory, binary, data) = setup();
    let (server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let paths = paths(directory.path());
    let mut send = raw(
        &client,
        directory.path(),
        "Everyone has arrived. Open the meeting.",
    );
    let receipt = accepted(&client, &mut send);
    let claim = claim_job(&client, send.claim_id);
    let mut first = preparation::prepare(
        &client,
        &paths,
        &claim,
        &tool(),
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    first.package.phrases[0].help.as_mut().unwrap().meaning =
        "Start an activity with other people.".into();
    client
        .post::<_, ()>("/api/v1/preparation/stage", &first)
        .unwrap();
    drop(server);
    rusqlite::Connection::open(data.join("phrase-gym.sqlite3"))
        .unwrap()
        .execute(
            "UPDATE inbox_generation SET lease_until = '2000-01-01T00:00:00Z' WHERE id = ?1",
            [receipt.generation_id.to_string()],
        )
        .unwrap();
    let (_server, _) = start(&binary, &data, url.rsplit(':').next().unwrap());
    let replacement = claim_job(&client, Uuid::new_v4());
    let second = preparation::prepare(
        &client,
        &paths,
        &replacement,
        &tool(),
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_ne!(first.package, second.package);
    assert_ne!(first.publication_id, second.publication_id);
    let unavailable = preparation::readiness::ClaudeTool {
        executable: directory.path().join("missing-tool"),
        host: phrase_gym_coach::adapter::environment::HostEnvironment::default(),
    };
    let recovered = preparation::prepare(
        &client,
        &paths,
        &replacement,
        &unavailable,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(recovered, second);
    assert_eq!(std::fs::read_dir(paths.attempts()).unwrap().count(), 2);
    assert!(
        client
            .post::<_, PublicationReceipt>("/api/v1/preparation/publish", &claim.fence)
            .unwrap_err()
            .starts_with("Server 409")
    );
    client
        .post::<_, ()>("/api/v1/preparation/stage", &second)
        .unwrap();
    let publication: PublicationReceipt = client
        .post("/api/v1/preparation/publish", &replacement.fence)
        .unwrap();
    assert_eq!(publication.generation_id, receipt.generation_id);
    assert_eq!(publication.target_ids.len(), 1);
    // A late identical sender retry reads the committed generation before attempting another model.
    let pending = directory.path().join("pending.json");
    assert_eq!(
        runner::deliver_with(
            &client,
            &pending,
            &mut send,
            |_| {},
            |_| panic!("Published generation must reconcile before any model call")
        )
        .unwrap(),
        publication
    );
    assert_retained_publication_state(&data);
}

fn claim_job(client: &ServerClient, claim_id: Uuid) -> JobClaim {
    client
        .post::<_, Option<JobClaim>>("/api/v1/preparation/claims", &ClaimRequest { claim_id })
        .unwrap()
        .unwrap()
}

fn assert_retained_publication_state(data: &Path) {
    let connection = rusqlite::Connection::open(data.join("phrase-gym.sqlite3")).unwrap();
    assert_eq!(
        connection
            .query_row::<i64, _, _>("SELECT COUNT(*) FROM target", [], |row| row.get(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row::<i64, _, _>("SELECT COUNT(*) FROM inbox_delivery", [], |row| row.get(0))
            .unwrap(),
        2
    );
}
