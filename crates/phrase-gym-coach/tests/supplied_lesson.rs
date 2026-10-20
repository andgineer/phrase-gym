use std::path::Path;

use phrase_gym_coach::supplied::{self, SuppliedLesson};
use phrase_gym_coach::transfer::{client, pending, runner};
use phrase_gym_contracts::inbox::JobClaim;
use phrase_gym_contracts::media::MediaAsset;
use serde_json::json;
use uuid::Uuid;

const LESSON: &str = include_str!("../../../fixtures/coach/supplied-lesson.json");

fn media(supplied: &SuppliedLesson) -> MediaAsset {
    MediaAsset {
        id: Uuid::new_v4(),
        sha256: "ab".repeat(32),
        byte_length: 100,
        mime_type: "audio/wav".into(),
        metadata: supplied.metadata.clone(),
        available: true,
    }
}

#[test]
fn supplied_content_binds_actual_accepted_media_and_server_pins_without_rewriting_text() {
    let supplied = SuppliedLesson::parse(LESSON).unwrap();
    let media = media(&supplied);
    let input = supplied.submission(Uuid::new_v4(), &media).unwrap();
    let generation_id = Uuid::new_v4();
    let claim: JobClaim = serde_json::from_value(json!({
        "fence": {"generation_id": generation_id, "claim_id": Uuid::new_v4(), "token": Uuid::new_v4(), "fence": 7},
        "lease_until": "2026-10-19T12:00:00Z", "deadline": "2026-10-19T12:08:00Z",
        "receipt": {"generation_id": generation_id, "submission_id": input.submission_id, "generation": 3, "namespace": "accepted-server-namespace", "profile_revision": 9, "guide_version": "pinned-guide", "received_at": "2026-10-19T11:58:00Z"},
        "input": input,
        "profile": {"revision": 9, "profile": {"native_language": "ru", "learning_timezone": "Europe/Belgrade", "languages": {}}, "mutation_id": null, "actor": "learner", "reason": "synthetic", "created_at": "2026-10-19T11:00:00Z", "restored_from": null},
        "guide": "The pinned preparation guide"
    })).unwrap();
    let output = supplied::output(&claim).unwrap();
    assert_eq!(output.fence, claim.fence);
    assert_eq!(output.package.profile_revision, 9);
    assert_eq!(output.package.namespace, "accepted-server-namespace");
    assert_eq!(output.package.source_revision, "3");
    assert_eq!(output.package.guide_version, "pinned-guide");
    let mut expected = supplied.phrase;
    expected.reference_audio_id = media.id;
    assert_eq!(output.package.phrases, [expected]);
    assert_eq!(
        supplied::output(&claim).unwrap().publication_id,
        output.publication_id
    );
    let mut changed = claim;
    changed.input.phrases[0].text = "Unrelated wording".into();
    assert!(supplied::output(&changed).is_err());
}

#[test]
fn pending_send_retains_exact_identity_and_audio_across_reload_and_failed_connection() {
    let directory = tempfile::tempdir().unwrap();
    let audio = directory.path().join("original.wav");
    let bytes = include_bytes!("../../../fixtures/preservation/media/silence-100ms.wav");
    std::fs::write(&audio, bytes).unwrap();
    let mut created = runner::create("http://127.0.0.1:1", LESSON, &audio).unwrap();
    let path = directory.path().join("pending.json");
    pending::save(&path, &created).unwrap();
    let reloaded = pending::load(&path).unwrap().unwrap();
    assert_eq!(created.submission_id, reloaded.submission_id);
    assert_eq!(created.upload, reloaded.upload);
    let client = client::ServerClient::new(
        "http://127.0.0.1:1",
        "synthetic-submit".into(),
        "synthetic-worker".into(),
    )
    .unwrap();
    assert!(runner::deliver(&client, &path, &mut created, |_| {}).is_err());
    let retry = pending::load(&path).unwrap().unwrap();
    assert_eq!(retry.submission_id, reloaded.submission_id);
    assert_eq!(retry.upload, reloaded.upload);
    let mut tampered = retry.clone();
    tampered.upload.audio_base64 = "dGFtcGVyZWQ=".into();
    assert!(
        tampered
            .verify(&client.server, client.identity)
            .unwrap_err()
            .contains("hash")
    );
    assert_eq!(
        pending::load(&path).unwrap().unwrap().upload,
        reloaded.upload
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn unsafe_connection_and_unsupported_input_fail_before_transfer() {
    for url in [
        "https://example.com",
        "http://example.com",
        "http://user:password@127.0.0.1",
        "http://127.0.0.1/path",
        "http://127.0.0.1?token=secret",
    ] {
        assert!(client::loopback(url).is_err(), "{url}");
    }
    assert_eq!(
        client::loopback("http://127.0.0.1:8000/").unwrap(),
        "http://127.0.0.1:8000"
    );
    assert!(SuppliedLesson::parse("Raw English text").is_err());
    assert!(SuppliedLesson::parse(&"x".repeat(8001)).is_err());
    let mut value: serde_json::Value = serde_json::from_str(LESSON).unwrap();
    value["metadata"]["text"] = json!("Unrelated reference");
    assert!(SuppliedLesson::parse(&value.to_string()).is_err());
    assert!(
        phrase_gym_coach::platform::files::read_selected(Path::new("/does-not-exist"), 100)
            .is_err()
    );
}
