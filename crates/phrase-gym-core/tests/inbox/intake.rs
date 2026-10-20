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
    assert!(
        inbox
            .receipt("sender", original.submission_id)
            .unwrap()
            .is_none()
    );
    let first = inbox.submit("sender", &original).unwrap();
    assert_eq!(
        inbox.receipt("sender", original.submission_id).unwrap(),
        Some(first.clone())
    );
    assert!(
        inbox
            .receipt("other", original.submission_id)
            .unwrap()
            .is_none()
    );
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
    assert_eq!(
        reopened.receipt("sender", original.submission_id).unwrap(),
        Some(first.clone())
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
    bad.phrases[0].audio_sha256 = Some("0".repeat(64));
    assert!(inbox.submit("sender", &bad).is_err());
    bad = original.clone();
    bad.phrases[0].reference_audio_id = Some(Uuid::new_v4());
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
            .join(original.phrases[0].audio_sha256.as_ref().unwrap()),
        b"truncated",
    )
    .unwrap();
    assert!(inbox.submit("sender", &original).is_err());
}

#[test]
fn missing_audio_is_durable_original_input_but_partial_or_invalid_supplied_audio_is_rejected() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let mut raw = input(&f);
    raw.phrases[0].reference_audio_id = None;
    assert!(inbox.submit("sender", &raw).is_err());
    raw.phrases[0].audio_sha256 = None;
    let receipt = inbox.submit("sender", &raw).unwrap();
    let claim = phrase_gym_core::inbox::jobs::Jobs::new(f.database.clone(), Arc::new(SystemClock))
        .claim("worker", Uuid::new_v4())
        .unwrap()
        .unwrap();
    assert_eq!(claim.input, raw);
    assert_eq!(claim.receipt, receipt);
    let mut substituted = raw.clone();
    substituted.phrases[0].reference_audio_id = Some(Uuid::new_v4());
    substituted.phrases[0].audio_sha256 = Some("0".repeat(64));
    assert!(inbox.submit("sender", &substituted).is_err());
    assert_eq!(
        inbox.receipt("sender", raw.submission_id).unwrap(),
        Some(receipt)
    );
}
