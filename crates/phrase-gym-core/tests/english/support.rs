use std::sync::Arc;

use base64::Engine;
use phrase_gym_contracts::english::PreparedEnglishLesson;
use phrase_gym_contracts::media::{MediaAsset, MediaUpload};
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::english::EnglishPackages;
use phrase_gym_core::media::ingest::MediaIngest;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::profiles::Profiles;
use phrase_gym_core::storage::Database;
use serde_json::{Value, json};
use tempfile::TempDir;

pub const ANSWER: &str = "Let's get started.";

pub struct Fixture {
    pub directory: TempDir,
    pub database: Arc<Database>,
    pub profiles: Arc<Profiles>,
    pub validator: EnglishPackages,
    ingest: MediaIngest,
}

impl Fixture {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let database = Arc::new(Database::open(directory.path()).unwrap());
        let store = Arc::new(MediaStore::open(directory.path()).unwrap());
        let profiles = Arc::new(Profiles::new(database.clone(), Arc::new(SystemClock)));
        profiles.initialize_current_learner().unwrap();
        Self {
            directory,
            database: database.clone(),
            profiles: profiles.clone(),
            validator: EnglishPackages::new(database.clone(), store.clone(), profiles),
            ingest: MediaIngest::new(database, store),
        }
    }

    pub fn upload(&self, overrides: &Value) -> MediaAsset {
        let mut metadata = json!({ "language": "en", "text": ANSWER, "role": "reference", "origin": "course", "source": "synthetic-fixture", "performer": "Synthetic", "locale": "en-US", "preparation_version": "fixture-v1" });
        for (key, value) in overrides.as_object().unwrap() {
            metadata[key] = value.clone();
        }
        let upload: MediaUpload = serde_json::from_value(json!({ "metadata": metadata, "mime_type": "audio/wav", "audio_base64": base64::engine::general_purpose::STANDARD.encode(include_bytes!("../../../../fixtures/preservation/media/silence-100ms.wav")) })).unwrap();
        self.ingest.upload(&upload).unwrap()
    }

    pub fn package(&self) -> PreparedEnglishLesson {
        let audio = self.upload(&json!({}));
        serde_json::from_value(json!({
            "schema_version": "english_phrases_v1", "namespace": "synthetic-lesson", "source_revision": "1", "language": "en", "profile_revision": 1, "guide_version": "fixture-guide-v1", "assessment_version": "situation-v1",
            "lesson": { "source_id": "meeting", "title": "Opening a meeting", "target_source_ids": ["open-meeting"] },
            "phrases": [{ "source_id": "open-meeting", "meaning_key": "begin-meeting", "intention_key": "open-meeting", "text": ANSWER, "accepted_variants": ["Shall we begin?"], "situations": [{ "id": "call", "text": "Everyone has joined the call. Open the meeting.", "language": "en", "audio_id": null, "partner": null, "exposed_target_source_ids": [] }], "hint": "started", "help": { "language": "en", "meaning": "A way to begin a shared activity.", "usage": null }, "details": { "language": "ru", "meaning": "Фраза, которой начинают совместное дело.", "translation": "Давайте начнём.", "usage": "Нейтральная фраза для начала встречи или урока.", "forms": null }, "familiarity": { "kind": "new" }, "reference_audio_id": audio.id, "expected_revision": null }]
        })).unwrap()
    }
}
