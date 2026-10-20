use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use phrase_gym_coach::supplied;
use phrase_gym_coach::transfer::{client::ServerClient, pending, runner};
use phrase_gym_contracts::inbox::{
    ClaimRequest, DeliveryStatus, JobClaim, LessonReceipt, LessonStatus, PublicationReceipt,
};
use phrase_gym_contracts::media::MediaAsset;
use uuid::Uuid;

const LESSON: &str = include_str!("../../../fixtures/coach/supplied-lesson.json");

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start(binary: &Path, data: &Path, port: &str) -> (Server, String) {
    let mut server = Server(
        Command::new(binary)
            .arg("serve")
            .arg("--data-dir")
            .arg(data)
            .args(["--port", port])
            .env("RUST_LOG", "info")
            .env("NO_COLOR", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let mut line = String::new();
    BufReader::new(server.0.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let url = line
        .split("http://")
        .nth(1)
        .expect("Server logs its actual loopback listener")
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != ':')
        .next()
        .unwrap();
    (server, format!("http://{url}"))
}

fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // This process boundary keeps Server and Coach out of each other's dependency graphs.
    assert!(
        Command::new(env!("CARGO"))
            .current_dir(&root)
            .args([
                "build",
                "--package",
                "phrase-gym-server",
                "--locked",
                "--offline"
            ])
            .status()
            .unwrap()
            .success()
    );
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!("phrase-gym-server{}", std::env::consts::EXE_SUFFIX));
    let directory = tempfile::tempdir().unwrap();
    let data = directory.path().join("server");
    let audio = directory.path().join("original.wav");
    std::fs::write(
        &audio,
        include_bytes!("../../../fixtures/preservation/media/silence-100ms.wav"),
    )
    .unwrap();
    (directory, binary, data)
}

fn accepted(client: &ServerClient, send: &mut pending::PendingSend) -> LessonReceipt {
    let media: MediaAsset = client.post("/api/v1/media", &send.upload).unwrap();
    send.submission = Some(
        send.material
            .submission(send.submission_id, &media)
            .unwrap(),
    );
    client
        .post("/api/v1/inbox/lessons", send.submission.as_ref().unwrap())
        .unwrap()
}

#[test]
fn production_server_accepts_real_coach_transfer_and_same_id_retry_after_restart() {
    let (directory, binary, data) = setup();
    let audio = directory.path().join("original.wav");
    let path = directory.path().join("pending.json");
    let (server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let mut send = runner::create(&url, LESSON, &audio).unwrap();
    let mut phases = Vec::new();
    let receipt = runner::deliver(&client, &path, &mut send, |phase| {
        phases.push(phase.to_owned());
    })
    .unwrap();
    assert!(!path.exists());
    assert_eq!(receipt.target_ids.len(), 1);
    assert!(
        phases
            .iter()
            .any(|phase| phase.starts_with("Accepted durably"))
    );
    // Retain the identical sender envelope as if the publication response had been lost.
    pending::save(&path, &send).unwrap();
    drop(server);
    let (_server, restarted_url) = start(&binary, &data, url.rsplit(':').next().unwrap());
    let restarted = ServerClient::new(&restarted_url).unwrap();
    assert_eq!(url, restarted_url);
    let recovered = runner::deliver(&restarted, &path, &mut send, |_| {}).unwrap();
    assert_eq!(receipt, recovered);
    assert!(!path.exists());
    let status: LessonStatus = restarted
        .get(&format!("/api/v1/inbox/lessons/{}", receipt.generation_id))
        .unwrap();
    assert_eq!(status.publication, Some(receipt));
}

#[test]
fn retry_moves_past_an_earlier_claim_without_publishing_or_losing_its_work() {
    let (directory, binary, data) = setup();
    let (_server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let audio = directory.path().join("original.wav");
    let mut earlier = runner::create(&url, LESSON, &audio).unwrap();
    let earlier_receipt = accepted(&client, &mut earlier);
    let mut json: serde_json::Value = serde_json::from_str(LESSON).unwrap();
    json["lesson"]["source_id"] = "second-supplied-lesson".into();
    let mut send = runner::create(&url, &json.to_string(), &audio).unwrap();
    let original_id = send.submission_id;
    let original_audio = send.upload.clone();
    let original_claim = send.claim_id;
    let path = directory.path().join("pending.json");
    assert!(
        runner::deliver(&client, &path, &mut send, |_| {})
            .unwrap_err()
            .contains("earlier inbox item")
    );
    let retained = pending::load(&path).unwrap().unwrap();
    assert_ne!(retained.claim_id, original_claim);
    assert_eq!(retained.submission_id, original_id);
    assert_eq!(retained.upload, original_audio);
    let earlier_status: LessonStatus = client
        .get(&format!(
            "/api/v1/inbox/lessons/{}",
            earlier_receipt.generation_id
        ))
        .unwrap();
    assert_eq!(earlier_status.status, DeliveryStatus::Preparing);
    assert!(earlier_status.publication.is_none());
    let claim: Option<JobClaim> = client
        .post(
            "/api/v1/preparation/claims",
            &ClaimRequest {
                claim_id: original_claim,
            },
        )
        .unwrap();
    let claim = claim.unwrap();
    assert_eq!(claim.receipt, earlier_receipt);
    // The worker responsible for the earlier item completes its own fenced output.
    client
        .post::<_, ()>(
            "/api/v1/preparation/stage",
            &supplied::output(&claim).unwrap(),
        )
        .unwrap();
    let earlier_publication: PublicationReceipt = client
        .post("/api/v1/preparation/publish", &claim.fence)
        .unwrap();
    let receipt = runner::deliver(&client, &path, &mut send, |_| {}).unwrap();
    assert_ne!(receipt.generation_id, earlier_publication.generation_id);
    assert_eq!(send.submission_id, original_id);
    assert!(!path.exists());
}

#[test]
fn superseded_attempt_receipt_conflict_rotates_only_the_attempt_before_retry() {
    let (directory, binary, data) = setup();
    let (server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let mut send = runner::create(&url, LESSON, &directory.path().join("original.wav")).unwrap();
    let receipt = accepted(&client, &mut send);
    let old_claim: Option<JobClaim> = client
        .post(
            "/api/v1/preparation/claims",
            &ClaimRequest {
                claim_id: send.claim_id,
            },
        )
        .unwrap();
    let old_claim = old_claim.unwrap();
    drop(server);
    // Expire the synthetic lease while stopped; production HTTP performs all replacement and fencing.
    rusqlite::Connection::open(data.join("phrase-gym.sqlite3"))
        .unwrap()
        .execute(
            "UPDATE inbox_generation SET lease_until = '2000-01-01T00:00:00Z' WHERE id = ?1",
            [receipt.generation_id.to_string()],
        )
        .unwrap();
    let (_server, _) = start(&binary, &data, url.rsplit(':').next().unwrap());
    let replacement: Option<JobClaim> = client
        .post(
            "/api/v1/preparation/claims",
            &ClaimRequest {
                claim_id: Uuid::new_v4(),
            },
        )
        .unwrap();
    let replacement = replacement.unwrap();
    assert!(replacement.fence.fence > old_claim.fence.fence);
    let path = directory.path().join("pending.json");
    let problem = runner::deliver(&client, &path, &mut send, |_| {}).unwrap_err();
    assert!(problem.starts_with("Server 409"), "{problem}");
    assert!(problem.contains("Unknown worker delivery"), "{problem}");
    let retained = pending::load(&path).unwrap().unwrap();
    assert_ne!(retained.claim_id, old_claim.fence.claim_id);
    assert_eq!(retained.submission_id, receipt.submission_id);
    assert_eq!(retained.submission, send.submission);
    client
        .post::<_, ()>(
            "/api/v1/preparation/stage",
            &supplied::output(&replacement).unwrap(),
        )
        .unwrap();
    let publication: PublicationReceipt = client
        .post("/api/v1/preparation/publish", &replacement.fence)
        .unwrap();
    assert_eq!(
        runner::deliver(&client, &path, &mut send, |_| {}).unwrap(),
        publication
    );
    assert!(!path.exists());
}

#[path = "support/generated_server.rs"]
mod generated_server;

#[path = "support/generated_recovery.rs"]
mod generated_recovery;

#[path = "support/private_socket.rs"]
mod private_socket;

#[path = "support/text_server.rs"]
mod text_server;
