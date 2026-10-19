use std::sync::Arc;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::inbox::DeliveryStatus;
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::inbox::Inbox;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::storage::Database;
use uuid::Uuid;

use super::{inbox, input, support::Fixture};

#[test]
fn durable_receipt_is_scoped_retry_safe_and_generations_are_immutable() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let original = input(&f);
    let first = inbox.submit("sender", &original).unwrap();
    assert_eq!(
        first.profile_revision,
        f.profiles.get(None).unwrap().revision
    );
    assert!(f.validator.validate(&f.package()).is_ok());
    assert_eq!(first, inbox.submit("sender", &original).unwrap());
    let mut changed = original.clone();
    changed.note = "A different note".into();
    assert!(inbox.submit("sender", &changed).is_err());
    changed.submission_id = Uuid::new_v4();
    assert!(inbox.submit("sender", &changed).is_err());
    changed.expected_generation = Some(1);
    let second = inbox.submit("sender", &changed).unwrap();
    assert_eq!(second.generation, 2);
    assert_eq!(second.namespace, first.namespace);
    assert_eq!(
        inbox.status("sender", first.generation_id).unwrap().status,
        DeliveryStatus::Superseded
    );
    assert!(inbox.status("other", first.generation_id).is_err());
    let reopened = Inbox::new(
        Arc::new(Database::open(f.directory.path()).unwrap()),
        Arc::new(MediaStore::open(f.directory.path()).unwrap()),
        Arc::new(SystemClock),
    );
    assert_eq!(first, reopened.submit("sender", &original).unwrap());
    assert_eq!(
        second,
        reopened
            .status("sender", second.generation_id)
            .unwrap()
            .receipt
    );
    assert_eq!(
        inbox.cancel("sender", second.generation_id).unwrap().status,
        DeliveryStatus::Cancelled
    );
}

#[test]
fn intake_rejects_unsupported_material_and_unverified_or_mismatched_audio() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let original = input(&f);
    let mut bad = original.clone();
    bad.phrases[0].audio_sha256 = "0".repeat(64);
    assert!(inbox.submit("sender", &bad).is_err());
    bad = original.clone();
    bad.phrases[0].reference_audio_id = Uuid::new_v4();
    assert!(inbox.submit("sender", &bad).is_err());
    bad = original.clone();
    bad.language = Language::De;
    assert!(inbox.submit("sender", &bad).is_err());
    bad = original.clone();
    bad.phrases = vec![bad.phrases[0].clone(); 6];
    assert!(inbox.submit("sender", &bad).is_err());
    bad = original.clone();
    bad.phrases[0].text = "https://example.test/source".into();
    assert!(inbox.submit("sender", &bad).is_err());
    bad = original.clone();
    bad.phrases[0].expected_revision = Some(1);
    assert!(inbox.submit("sender", &bad).is_err());
    std::fs::write(
        f.directory
            .path()
            .join("media")
            .join(&original.phrases[0].audio_sha256),
        b"truncated",
    )
    .unwrap();
    assert!(inbox.submit("sender", &original).is_err());
}
