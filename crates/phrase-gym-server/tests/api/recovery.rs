use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::http::StatusCode;
use phrase_gym_core::clock::Clock;
use phrase_gym_core::hashing::sha256_hex;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::recovery::{backup, restore};
use phrase_gym_core::storage::Database;
use phrase_gym_core::timestamps;
use phrase_gym_server::{AppState, router};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::content::{WAV, get, package, post, publish, stage, upload};
use crate::support::documents::base64;
use crate::support::practice::{apply, count, moment, ready, session};
use crate::support::{TestServer, repository_root};

#[tokio::test]
async fn isolated_restore_preserves_library_settings_history_sessions_all_takes_and_staged_audio() {
    let (server, state) = crate::support::capture::due().await;
    let audio = std::fs::read(repository_root().join(WAV)).unwrap();
    let attempt = &state["attempts"][0];
    let mut takes = Vec::new();
    for complete in [true, false] {
        let take = json!({
            "id": Uuid::new_v4(), "target_id": attempt["target_id"], "revision": attempt["revision"],
            "attempt_id": attempt["id"], "captured_at": timestamps::isoformat(server.clock.now()),
            "mime_type": "audio/wav", "complete": complete, "audio_base64": base64(&audio),
        });
        let saved = post(&server.router, "/api/v1/recordings", &take).await;
        assert_eq!(saved.status, StatusCode::OK, "{:?}", saved.body);
        takes.push((take, saved.json()));
    }
    let (_, result) = apply(&server, &state, "rate", &json!({"rating": "recalled"})).await;
    let saved_session = result["session"].clone();
    let settings = json!({"native_language": "sr", "ui_language_mode": "learning"});
    let connection = rusqlite::Connection::open(&server.database_path).unwrap();
    connection
        .execute(
            "INSERT INTO device_settings VALUES (1, ?1)",
            [settings.to_string()],
        )
        .unwrap();
    let staged_audio = stage_after_revision(&server).await;
    let root = tempfile::tempdir().unwrap();
    let snapshot = root.path().join("snapshot");
    let manifest = backup(&server.data_dir, &snapshot).unwrap();
    assert_eq!(manifest.media.len(), 2);
    connection
        .execute("UPDATE device_settings SET payload = '{}'", [])
        .unwrap();
    let isolated = root.path().join("isolated");
    restore(&snapshot, &isolated).unwrap();
    let database = Database::open(&isolated).unwrap();
    for table in [
        "target",
        "target_revision",
        "staged_package",
        "target_schedule",
        "chunk_history",
        "review_receipt",
        "practice_event",
        "practice_attempt",
        "learner_recording",
    ] {
        let restored: i64 = database
            .read(|tx| tx.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)))
            .unwrap();
        assert_eq!(restored, count(&server, table), "{table}");
        assert!(restored > 0, "{table}");
    }
    let store = MediaStore::open(&isolated).unwrap();
    std::fs::remove_file(server.data_dir.join("media").join(sha256_hex(&audio))).unwrap();
    let app = router(
        AppState::new(Arc::new(database), Arc::new(store), server.clock.clone()),
        &crate::support::settings(),
    );
    assert_eq!(get(&app, "/api/v1/settings").await.json(), settings);
    assert_eq!(session(&app, &saved_session["id"]).await, saved_session);
    for (take, receipt) in takes {
        assert_eq!(
            post(&app, "/api/v1/recordings", &take).await.json(),
            receipt
        );
        let played = get(
            &app,
            &format!("/api/v1/recordings/{}", take["id"].as_str().unwrap()),
        )
        .await;
        assert_eq!(played.status, StatusCode::OK);
        assert_eq!(played.body.as_ref(), audio);
    }
    for id in saved_session["manifest"]["required_media_ids"]
        .as_array()
        .unwrap()
    {
        let played = get(&app, &format!("/api/v1/media/{}", id.as_str().unwrap())).await;
        assert_eq!(played.status, StatusCode::OK);
        assert_eq!(played.body.as_ref(), audio);
    }
    let played = get(
        &app,
        &format!("/api/v1/media/{}", staged_audio["id"].as_str().unwrap()),
    )
    .await;
    assert_eq!(
        played.body.as_ref(),
        std::fs::read(repository_root().join("fixtures/preservation/media/tone.mp3")).unwrap()
    );
    assert_eq!(
        get(&app, "/api/v1/phrases/search?language=en&q=")
            .await
            .json(),
        get(&server.router, "/api/v1/phrases/search?language=en&q=")
            .await
            .json()
    );
}

async fn stage_after_revision(server: &TestServer) -> Value {
    let staged_audio = upload(
        &server.router,
        &json!({"text": package()["targets"][0]["text"]}),
        "fixtures/preservation/media/tone.mp3",
        "audio/mpeg",
    )
    .await;
    let mut revised = package();
    for target in revised["targets"].as_array_mut().unwrap() {
        let asset = upload(
            &server.router,
            &json!({"text": target["text"]}),
            "fixtures/preservation/media/tone.mp3",
            "audio/mpeg",
        )
        .await;
        target["reference_audio_id"] = asset["id"].clone();
    }
    publish(&server.router, &revised).await;
    let mut pending = package();
    pending["namespace"] = json!("recovery.staged");
    pending["targets"][0]["reference_audio_id"] = staged_audio["id"].clone();
    let staged = stage(&server.router, &pending).await;
    assert_eq!(staged.status, StatusCode::OK);
    assert_eq!(staged.json()["status"], "staged", "{:?}", staged.body);
    staged_audio
}

#[tokio::test]
async fn reconciliation_reports_damage_and_orphans_and_retains_old_staging_and_backup_assets() {
    let server = TestServer::start(moment("2026-10-12T12:00:00Z"));
    let targets = ready(&server.router, &mut package()).await;
    let media = server.data_dir.join("media");
    let audio = std::fs::read(repository_root().join(WAV)).unwrap();
    let original = media.join(sha256_hex(&audio));
    std::fs::write(media.join("orphan"), b"unreferenced original").unwrap();
    let staging = server.data_dir.join("staging").join("old.part");
    std::fs::write(&staging, b"possibly active staged transfer").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&staging)
        .unwrap()
        .set_times(
            std::fs::FileTimes::new().set_modified(SystemTime::now() - Duration::from_hours(2)),
        )
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    let snapshot = root.path().join("backup");
    backup(&server.data_dir, &snapshot).unwrap();
    for damaged in [true, false] {
        if damaged {
            std::fs::write(&original, vec![0; audio.len()]).unwrap();
        } else {
            std::fs::remove_file(&original).unwrap();
        }
        let report = post(&server.router, "/api/v1/media/reconcile", &json!({}))
            .await
            .json();
        assert_eq!(report["removed_staging_files"], 0);
        assert_eq!(report["orphan_hashes"], json!(["orphan"]));
        assert_eq!(
            report["missing_ids"].as_array().unwrap().len(),
            targets.len()
        );
        for target in &targets {
            let id = target["content"]["reference_audio_id"].as_str().unwrap();
            assert_eq!(
                get(&server.router, &format!("/api/v1/media/{id}"))
                    .await
                    .status,
                StatusCode::SERVICE_UNAVAILABLE
            );
            assert_eq!(
                get(&server.router, &format!("/api/v1/media/{id}/metadata"))
                    .await
                    .json()["available"],
                false
            );
        }
        assert!(staging.exists());
        assert_eq!(
            std::fs::read(media.join("orphan")).unwrap(),
            b"unreferenced original"
        );
        phrase_gym_core::recovery::verify(&snapshot).unwrap();
    }
    let restored = upload(
        &server.router,
        &json!({"text": package()["targets"][0]["text"]}),
        WAV,
        "audio/wav",
    )
    .await;
    assert_eq!(restored["available"], true);
    assert!(
        targets
            .iter()
            .any(|target| target["content"]["reference_audio_id"] == restored["id"])
    );
    assert!(
        post(&server.router, "/api/v1/media/reconcile", &json!({}))
            .await
            .json()["missing_ids"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn recovery_commands_roundtrip_and_reject_existing_destinations() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    let _database = Database::open(&live).unwrap();
    let snapshot = root.path().join("snapshot");
    let isolated = root.path().join("isolated");
    let command = |name: &str, arguments: &[&std::path::Path]| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_phrase-gym-server"));
        command.arg(name);
        for pair in arguments.chunks_exact(2) {
            command.arg(pair[0]).arg(pair[1]);
        }
        command.output().unwrap()
    };
    let data_flag = std::path::Path::new("--data-dir");
    let snapshot_flag = std::path::Path::new("--snapshot");
    for (name, args) in [
        (
            "backup",
            vec![data_flag, live.as_path(), snapshot_flag, snapshot.as_path()],
        ),
        ("verify-backup", vec![snapshot_flag, snapshot.as_path()]),
        (
            "restore",
            vec![
                snapshot_flag,
                snapshot.as_path(),
                data_flag,
                isolated.as_path(),
            ],
        ),
    ] {
        let output = command(name, &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(Database::open(&isolated).is_ok());
    assert!(
        !command("restore", &[snapshot_flag, &snapshot, data_flag, &live])
            .status
            .success()
    );
}
