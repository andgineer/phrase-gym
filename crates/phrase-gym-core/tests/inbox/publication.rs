use std::sync::Arc;

use phrase_gym_contracts::inbox::DeliveryStatus;
use phrase_gym_core::inbox::{delivery::Delivery, jobs::Jobs};
use uuid::Uuid;

use super::{TestClock, inbox, input, output, support::Fixture};

#[test]
fn staged_publication_is_atomic_and_committed_receipts_recover_after_expiry() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let receipt = inbox.submit("sender", &input(&f)).unwrap();
    let clock = Arc::new(TestClock::new());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let output = output(&f, &claim);
    let delivery = Delivery::new(
        f.database.clone(),
        Arc::new(phrase_gym_core::english::EnglishPackages::new(
            f.database.clone(),
            Arc::new(phrase_gym_core::media::store::MediaStore::open(f.directory.path()).unwrap()),
            f.profiles.clone(),
        )),
        clock.clone(),
    );
    delivery.stage("worker", &output).unwrap();
    assert_eq!(
        inbox
            .status("sender", receipt.generation_id)
            .unwrap()
            .status,
        DeliveryStatus::PendingDelivery
    );
    f.database.write(|tx| tx.execute_batch("CREATE TRIGGER fail_publication BEFORE INSERT ON target_revision BEGIN SELECT RAISE(ABORT, 'injected'); END;")).unwrap();
    assert!(delivery.publish("worker", &claim.fence).is_err());
    let count: i64 = f
        .database
        .read(|tx| tx.query_row("SELECT COUNT(*) FROM target", [], |row| row.get(0)))
        .unwrap();
    assert_eq!(count, 0);
    assert!(delivery.receipt("worker", &claim.fence).unwrap().is_none());
    f.database
        .write(|tx| tx.execute_batch("DROP TRIGGER fail_publication"))
        .unwrap();
    let published = delivery.publish("worker", &claim.fence).unwrap();
    clock.advance(1000);
    assert_eq!(published, delivery.publish("worker", &claim.fence).unwrap());
    delivery.stage("worker", &output).unwrap();
    let mut conflict = output.clone();
    conflict.package.lesson.title = "Different title".into();
    assert!(delivery.stage("worker", &conflict).is_err());
    assert!(delivery.publish("other", &claim.fence).is_err());
    assert_eq!(
        inbox
            .status("sender", receipt.generation_id)
            .unwrap()
            .publication,
        Some(published)
    );
}

#[test]
fn stale_workers_and_changed_pinned_input_cannot_publish() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let mut input = input(&f);
    inbox.submit("sender", &input).unwrap();
    let clock = Arc::new(TestClock::new());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let mut output = output(&f, &claim);
    let delivery = Delivery::new(
        f.database.clone(),
        Arc::new(phrase_gym_core::english::EnglishPackages::new(
            f.database.clone(),
            Arc::new(phrase_gym_core::media::store::MediaStore::open(f.directory.path()).unwrap()),
            f.profiles.clone(),
        )),
        clock.clone(),
    );
    output.package.profile_revision = 99;
    assert!(delivery.stage("worker", &output).is_err());
    output.package.profile_revision = claim.receipt.profile_revision;
    output.package.phrases[0].reference_audio_id = Uuid::new_v4();
    assert!(delivery.stage("worker", &output).is_err());
    output.package.phrases[0].reference_audio_id = input.phrases[0].reference_audio_id.unwrap();
    delivery.stage("worker", &output).unwrap();
    clock.advance(120);
    assert!(delivery.publish("worker", &claim.fence).is_err());
    let fresh = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    assert!(delivery.publish("worker", &claim.fence).is_err());
    input.submission_id = Uuid::new_v4();
    input.expected_generation = Some(1);
    inbox.submit("sender", &input).unwrap();
    assert!(
        delivery
            .stage("worker", &super::output(&f, &fresh))
            .is_err()
    );
}

#[test]
fn isolated_restore_preserves_input_staged_media_guides_and_fences_captured_leases() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let original = input(&f);
    let receipt = inbox.submit("sender", &original).unwrap();
    let clock = Arc::new(TestClock::new());
    let claim = Jobs::new(f.database.clone(), clock.clone())
        .claim("worker", Uuid::new_v4())
        .unwrap()
        .unwrap();
    let store =
        Arc::new(phrase_gym_core::media::store::MediaStore::open(f.directory.path()).unwrap());
    let delivery = Delivery::new(
        f.database.clone(),
        Arc::new(phrase_gym_core::english::EnglishPackages::new(
            f.database.clone(),
            store,
            f.profiles.clone(),
        )),
        clock.clone(),
    );
    delivery.stage("worker", &output(&f, &claim)).unwrap();
    let root = tempfile::tempdir().unwrap();
    let snapshot = root.path().join("snapshot");
    let restored = root.path().join("restored");
    let manifest = phrase_gym_core::recovery::backup(f.directory.path(), &snapshot).unwrap();
    assert_eq!(manifest.media.len(), 1);
    phrase_gym_core::recovery::restore(&snapshot, &restored).unwrap();
    let database = Arc::new(phrase_gym_core::storage::Database::open(&restored).unwrap());
    let restored_inbox = phrase_gym_core::inbox::Inbox::new(
        database.clone(),
        Arc::new(phrase_gym_core::media::store::MediaStore::open(&restored).unwrap()),
        clock.clone(),
    );
    assert_eq!(restored_inbox.submit("sender", &original).unwrap(), receipt);
    let jobs = Jobs::new(database, clock);
    assert!(jobs.renew("worker", &claim.fence).is_err());
    let fresh = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    assert_eq!(fresh.input, original);
    assert_eq!(fresh.guide, claim.guide);
    assert_eq!(fresh.profile, claim.profile);
    assert_ne!(fresh.fence.token, claim.fence.token);
}

#[test]
fn compatible_generations_keep_target_identity_and_expected_revisions_are_rechecked() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let clock = Arc::new(TestClock::new());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let store =
        Arc::new(phrase_gym_core::media::store::MediaStore::open(f.directory.path()).unwrap());
    let delivery = Delivery::new(
        f.database.clone(),
        Arc::new(phrase_gym_core::english::EnglishPackages::new(
            f.database.clone(),
            store,
            f.profiles.clone(),
        )),
        clock,
    );
    let mut original = input(&f);
    inbox.submit("sender", &original).unwrap();
    let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let first = output(&f, &claim);
    delivery.stage("worker", &first).unwrap();
    let published = delivery.publish("worker", &claim.fence).unwrap();
    original.submission_id = Uuid::new_v4();
    original.expected_generation = Some(1);
    assert!(inbox.submit("sender", &original).is_err());
    original.phrases[0].expected_revision = Some(1);
    inbox.submit("sender", &original).unwrap();
    let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let mut second = output(&f, &claim);
    second.package.phrases[0].expected_revision = Some(1);
    second.package.phrases[0].meaning_key = "changed-meaning".into();
    assert!(delivery.stage("worker", &second).is_err());
    second.package.phrases[0]
        .meaning_key
        .clone_from(&first.package.phrases[0].meaning_key);
    delivery.stage("worker", &second).unwrap();
    let second_receipt = delivery.publish("worker", &claim.fence).unwrap();
    assert_eq!(published.target_ids, second_receipt.target_ids);
    original.submission_id = Uuid::new_v4();
    original.expected_generation = Some(2);
    original.phrases[0].expected_revision = Some(2);
    inbox.submit("sender", &original).unwrap();
    let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let mut third = output(&f, &claim);
    third.package.phrases[0].expected_revision = Some(2);
    delivery.stage("worker", &third).unwrap();
    f.database
        .write(|tx| tx.execute("UPDATE target SET current_revision = 3", []))
        .unwrap();
    assert!(delivery.publish("worker", &claim.fence).is_err());
    assert!(delivery.receipt("worker", &claim.fence).unwrap().is_none());
}

#[test]
fn genuinely_missing_reference_requires_generated_provenance_without_rebinding_original_input() {
    let f = Fixture::new();
    let inbox = inbox(&f);
    let mut raw = input(&f);
    raw.phrases[0].reference_audio_id = None;
    raw.phrases[0].audio_sha256 = None;
    inbox.submit("sender", &raw).unwrap();
    let clock = Arc::new(TestClock::new());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let mut output = output(&f, &claim);
    let delivery = Delivery::new(
        f.database.clone(),
        Arc::new(phrase_gym_core::english::EnglishPackages::new(
            f.database.clone(),
            Arc::new(phrase_gym_core::media::store::MediaStore::open(f.directory.path()).unwrap()),
            f.profiles.clone(),
        )),
        clock,
    );
    assert!(
        delivery.stage("worker", &output).is_err(),
        "Course/actor media cannot masquerade as generated speech"
    );
    let generated = f.upload(&serde_json::json!({"origin":"generated", "performer":null, "engine":"synthetic", "voice":"synthetic", "speech_text":super::support::ANSWER}));
    output.package.phrases[0].reference_audio_id = generated.id;
    delivery.stage("worker", &output).unwrap();
    delivery.publish("worker", &claim.fence).unwrap();
    assert_eq!(
        jobs.claim("worker", claim.fence.claim_id)
            .unwrap()
            .unwrap()
            .input,
        raw
    );
}
