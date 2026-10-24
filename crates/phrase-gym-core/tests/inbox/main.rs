mod identity;
mod intake;
mod jobs;
mod learning;
mod new_limit;
mod publication;
mod rotation;
mod study;
#[path = "../english/support.rs"]
mod support;
mod voluntary;

use std::sync::Arc;

use phrase_gym_contracts::inbox::{LessonPhraseInput, LessonSubmission};
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::inbox::Inbox;
use phrase_gym_core::media::store::MediaStore;
use serde_json::json;
use uuid::Uuid;

fn inbox(f: &support::Fixture) -> Inbox {
    Inbox::new(
        f.database.clone(),
        Arc::new(MediaStore::open(f.directory.path()).unwrap()),
        Arc::new(SystemClock),
    )
}

fn input(f: &support::Fixture) -> LessonSubmission {
    let audio = f.upload(&json!({}));
    LessonSubmission {
        submission_id: Uuid::new_v4(),
        item_id: "meeting".into(),
        expected_generation: None,
        language: phrase_gym_contracts::base::Language::En,
        title: "Meeting".into(),
        context: "A meeting opener".into(),
        note: String::new(),
        phrases: vec![LessonPhraseInput {
            source_id: "open-meeting".into(),
            text: support::ANSWER.into(),
            reference_audio_id: Some(audio.id),
            audio_sha256: Some(audio.sha256),
            expected_revision: None,
        }],
    }
}

struct TestClock(std::sync::Mutex<chrono::DateTime<chrono::Utc>>);
impl TestClock {
    fn new() -> Self {
        Self(std::sync::Mutex::new(
            "2026-10-19T10:00:00Z".parse().unwrap(),
        ))
    }
    fn advance(&self, seconds: i64) {
        *self.0.lock().unwrap() += chrono::Duration::seconds(seconds);
    }
}
impl phrase_gym_core::clock::Clock for TestClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        *self.0.lock().unwrap()
    }
}

fn output(
    f: &support::Fixture,
    claim: &phrase_gym_contracts::inbox::JobClaim,
) -> phrase_gym_contracts::inbox::LessonOutput {
    let mut package = f.package();
    package.namespace.clone_from(&claim.receipt.namespace);
    package.source_revision = claim.receipt.generation.to_string();
    package
        .guide_version
        .clone_from(&claim.receipt.guide_version);
    package.profile_revision = claim.receipt.profile_revision;
    phrase_gym_contracts::inbox::LessonOutput {
        fence: claim.fence.clone(),
        publication_id: Uuid::new_v4(),
        adapter: "fake-worker-v1".into(),
        package,
    }
}
