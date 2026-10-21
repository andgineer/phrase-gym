use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::content::ContentPackage;
use phrase_gym_contracts::media::{MediaUpload, ReferenceMime};
use phrase_gym_contracts::sessions::SessionStart;
use uuid::Uuid;

use super::{Sessions, Started};
use crate::clock::ManualClock;
use crate::content::staging::PackageStaging;
use crate::error::CoreError;
use crate::media::index::MediaIndex;
use crate::media::ingest::MediaIngest;
use crate::media::store::MediaStore;
use crate::storage::Database;
use crate::timestamps;

fn repository_file(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn published(database: &Arc<Database>, store: &Arc<MediaStore>, clock: &Arc<ManualClock>) {
    let audio = std::fs::read(repository_file(
        "fixtures/preservation/media/silence-100ms.wav",
    ))
    .unwrap();
    let encoded = base64_text(&audio);
    let text = std::fs::read_to_string(repository_file(
        "fixtures/preservation/packages/en-basic.json",
    ))
    .unwrap();
    let mut package: ContentPackage = serde_json::from_str(&text).unwrap();
    let ingest = MediaIngest::new(database.clone(), store.clone());
    for target in &mut package.targets {
        let upload: MediaUpload = serde_json::from_value(serde_json::json!({
            "metadata": {"language": "en", "text": target.text, "role": "reference",
                         "origin": "test", "source": "unit-test", "locale": "en-US",
                         "preparation_version": "1"},
            "mime_type": ReferenceMime::Wav,
            "audio_base64": encoded,
        }))
        .unwrap();
        target.reference_audio_id = Some(ingest.upload(&upload).unwrap().id);
    }
    let staging = PackageStaging::new(database.clone(), store.clone(), clock.clone());
    let report = staging.stage(&package).unwrap();
    staging.publish(report.id.unwrap()).unwrap();
}

fn base64_text(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

#[test]
fn a_start_never_plans_with_media_it_did_not_check() {
    let directory = tempfile::tempdir().unwrap();
    let database = Arc::new(Database::open(directory.path()).unwrap());
    let store = Arc::new(MediaStore::open(directory.path()).unwrap());
    let clock = Arc::new(ManualClock::new(
        timestamps::parse("2026-10-12T12:00:00Z").unwrap(),
    ));
    published(&database, &store, &clock);
    let sessions = Sessions::new(database.clone(), store, clock);
    let start = SessionStart {
        voluntary: false,
        id: Uuid::new_v4(),
        language: Language::En,
        lesson_id: None,
        dialogue_id: None,
        takeover_session_id: None,
    };
    let unchecked = MediaIndex::default();
    let retry = database
        .write(|transaction| sessions.begin(transaction, &start, "{}", &unchecked, false))
        .unwrap();
    assert!(matches!(retry, Started::NeedsMedia(ids) if ids.len() == 3));
    assert!(matches!(sessions.current(), Ok(None)));
    let last =
        database.write(|transaction| sessions.begin(transaction, &start, "{}", &unchecked, true));
    assert!(
        matches!(last, Err(CoreError::Conflict(_))),
        "unchecked media counts as unavailable"
    );
    assert_eq!(sessions.start(&start).unwrap().manifest.targets.len(), 3);
}
