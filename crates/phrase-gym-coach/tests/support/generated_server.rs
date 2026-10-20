use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use phrase_gym_coach::adapter::environment::HostEnvironment;
use phrase_gym_coach::platform::paths::CoachPaths;
use phrase_gym_coach::preparation::{self, readiness::ClaudeTool};
use phrase_gym_coach::transfer::material::{Material, RawLesson};

use super::*;

pub(super) fn paths(directory: &Path) -> CoachPaths {
    let paths = CoachPaths {
        config: directory.join("config"),
        cache: directory.join("cache"),
        logs: directory.join("logs"),
        state: directory.join("state"),
    };
    paths.ensure().unwrap();
    paths
}

pub(super) fn tool() -> ClaudeTool {
    ClaudeTool {
        executable: PathBuf::from(env!("CARGO_BIN_EXE_phrase-gym-fake-child")),
        host: HostEnvironment::default(),
    }
}

pub(super) fn raw(client: &ServerClient, directory: &Path, context: &str) -> pending::PendingSend {
    runner::create_material(
        &client.server,
        Material::Raw(RawLesson {
            title: "Opening a meeting".into(),
            phrase: "Let's get started.".into(),
            situation: context.into(),
            performer: "Synthetic".into(),
            locale: "en-US".into(),
        }),
        &directory.join("original.wav"),
    )
    .unwrap()
}

#[test]
fn raw_input_runs_native_fake_claude_and_reconciles_committed_result_after_restart() {
    let (directory, binary, data) = setup();
    let (server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let paths = paths(directory.path());
    let file = directory.path().join("raw-pending.json");
    let mut send = raw(
        &client,
        directory.path(),
        "Open a meeting after everyone arrives.",
    );
    let original = send.upload.clone();
    let publication = runner::deliver_with(
        &client,
        &file,
        &mut send,
        |_| {},
        |claim| {
            preparation::prepare(
                &client,
                &paths,
                claim,
                &tool(),
                &Arc::new(AtomicBool::new(false)),
            )
        },
    )
    .unwrap();
    assert_eq!(send.upload, original);
    assert_eq!(
        send.output.as_ref().unwrap().package.phrases[0].text,
        "Let's get started."
    );
    assert_eq!(std::fs::read_dir(paths.attempts()).unwrap().count(), 1);
    pending::save(&file, &send).unwrap();
    drop(server);
    let (_server, _) = start(&binary, &data, url.rsplit(':').next().unwrap());
    let repeated = runner::deliver_with(
        &client,
        &file,
        &mut send,
        |_| {},
        |_| panic!("A committed attempt must never call the model again"),
    )
    .unwrap();
    assert_eq!(repeated, publication);
    assert_eq!(std::fs::read_dir(paths.attempts()).unwrap().count(), 1);
}

#[test]
fn malformed_overflow_and_cancelled_processes_publish_nothing_and_preserve_original_input() {
    for (context, expected, cancel_after) in [
        ("fake-invalid-hint", "validation", None),
        ("fake-auth", "subscription login", None),
        ("fake-quota", "quota", None),
        ("fake-malformed", "malformed", None),
        ("fake-flood", "64 KiB", None),
        ("fake-prefix-flood", "64 KiB", None),
        ("fake-wait", "cancelled", Some(Duration::from_millis(200))),
    ] {
        let (directory, binary, data) = setup();
        let (_server, url) = start(&binary, &data, "0");
        let client = ServerClient::new(&url).unwrap();
        let paths = paths(directory.path());
        let file = directory.path().join("raw-pending.json");
        let mut send = raw(&client, directory.path(), context);
        let original = send.upload.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&cancel);
        let timer = cancel_after.map(|duration| {
            std::thread::spawn(move || {
                std::thread::sleep(duration);
                stop.store(true, Ordering::SeqCst);
            })
        });
        let problem = runner::deliver_with(
            &client,
            &file,
            &mut send,
            |_| {},
            |claim| preparation::prepare(&client, &paths, claim, &tool(), &cancel),
        )
        .unwrap_err();
        if let Some(timer) = timer {
            timer.join().unwrap();
        }
        assert!(problem.contains(expected), "{problem}");
        let retained = pending::load(&file).unwrap().unwrap();
        assert_eq!(retained.submission_id, send.submission_id);
        assert_eq!(retained.upload, original);
        let receipt: Option<LessonReceipt> = client
            .get(&format!("/api/v1/inbox/submissions/{}", send.submission_id))
            .unwrap();
        let status: LessonStatus = client
            .get(&format!(
                "/api/v1/inbox/lessons/{}",
                receipt.unwrap().generation_id
            ))
            .unwrap();
        assert!(status.publication.is_none());
        assert_eq!(status.status, DeliveryStatus::Failed);
        runner::deliver_with(
            &client,
            &file,
            &mut send,
            |_| {},
            |_| panic!("Failure must require an explicit preparation retry"),
        )
        .unwrap_err();
    }
}

#[test]
fn a_pending_envelope_never_transfers_under_changed_server_identity() {
    let (directory, binary, data) = setup();
    let (_server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let mut send = raw(&client, directory.path(), "Open the meeting.");
    send.identity = Some(Uuid::new_v4());
    let path = directory.path().join("pending.json");
    let problem =
        runner::deliver_with(&client, &path, &mut send, |_| {}, |_| panic!()).unwrap_err();
    assert!(problem.contains("original Server account"));
    assert!(!path.exists());
}

#[test]
fn cancellation_after_native_start_fences_even_a_valid_zero_exit_result() {
    let (directory, binary, data) = setup();
    let (_server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let paths = paths(directory.path());
    let mut send = raw(&client, directory.path(), "fake-lease-loss");
    let accepted = accepted(&client, &mut send);
    let attempts = paths.attempts();
    let actor_url = url.clone();
    let generation = accepted.generation_id;
    let cancel = std::thread::spawn(move || {
        let started = std::time::Instant::now();
        let directory = loop {
            if let Some(path) = std::fs::read_dir(&attempts)
                .ok()
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path())
                .find(|path| path.join("running.marker").exists())
            {
                break path;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "Fake native process never started"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        let actor = ServerClient::new(&actor_url).unwrap();
        actor
            .post::<_, LessonStatus>(&format!("/api/v1/inbox/lessons/{generation}/cancel"), &())
            .unwrap();
        std::fs::write(directory.join("release.signal"), b"continue").unwrap();
    });
    let path = directory.path().join("pending.json");
    let error = runner::deliver_with(
        &client,
        &path,
        &mut send,
        |_| {},
        |claim| {
            preparation::prepare(
                &client,
                &paths,
                claim,
                &tool(),
                &Arc::new(AtomicBool::new(false)),
            )
        },
    )
    .unwrap_err();
    cancel.join().unwrap();
    assert!(error.starts_with("Server 409"), "{error}");
    let status: LessonStatus = client
        .get(&format!("/api/v1/inbox/lessons/{generation}"))
        .unwrap();
    assert_eq!(status.status, DeliveryStatus::Cancelled);
    assert!(status.publication.is_none());
    assert_eq!(
        pending::load(&path).unwrap().unwrap().submission_id,
        send.submission_id
    );
}
