use std::sync::Arc;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::practice::{PracticeKind, RecallRating};
use phrase_gym_contracts::sessions::{Eligibility, SessionStart};
use phrase_gym_core::clock::Clock;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::scheduler::ReviewPolicy;
use phrase_gym_core::timestamps;
use uuid::Uuid;

use super::learning::{Learning, publish};

#[test]
fn completed_study_enables_later_session_recall_without_inventing_memory() {
    let learning = Learning::new();
    let before = learning.readiness.summary(Language::En, None).unwrap();
    assert_eq!((before.new_count, before.due_count), (1, 0));
    let first = learning.start(None);
    learning.event(first.id, 1, PracticeKind::Present).unwrap();
    let unfinished = learning.sessions.get(first.id).unwrap();
    let study = learning.start(Some(first.id));
    assert_eq!(study.manifest.targets[0].eligibility, Eligibility::Study);
    learning.event(study.id, 1, PracticeKind::Present).unwrap();
    let completion = learning.event(study.id, 2, PracticeKind::Next).unwrap();
    assert_eq!(
        completion.receipts[0].eligibility_reason.as_deref(),
        Some("ungraded_study")
    );
    assert!(!completion.receipts[0].schedule_changed);
    assert!(completion.session.attempts[0].original.is_none());
    assert_eq!(learning.schedule_count(), 0);
    assert_eq!(
        learning
            .readiness
            .summary(Language::En, None)
            .unwrap()
            .new_count,
        1
    );
    assert_eq!(
        learning.sessions.get(first.id).unwrap().attempts,
        unfinished.attempts
    );

    let recall = learning.start(None);
    assert_eq!(recall.manifest.targets[0].eligibility, Eligibility::New);
    assert_eq!(recall.manifest.targets[0].hidden_chunks, vec!["phrase"]);
    let presented = learning.event(recall.id, 1, PracticeKind::Present).unwrap();
    assert!(!presented.session.attempts[0].revealed);
    assert!(!presented.session.attempts[0].supported);
    assert!(learning.event(recall.id, 2, PracticeKind::Rate).is_err());
    let revealed = learning.event(recall.id, 2, PracticeKind::Answer).unwrap();
    let original = revealed.session.attempts[0].original.clone().unwrap();
    assert!(!original.supported);
    let rated = learning.event(recall.id, 3, PracticeKind::Rate).unwrap();
    assert_eq!(rated.session.attempts[0].original.as_ref(), Some(&original));
    assert_eq!(
        rated.receipts[0].eligibility_reason.as_deref(),
        Some("eligible")
    );
    assert!(rated.receipts[0].schedule_changed);
    let expected = ReviewPolicy::default()
        .review(None, RecallRating::Recalled, learning.clock.now())
        .unwrap();
    let (version, due, memory): (i64, String, String) = learning
        .fixture
        .database
        .read(|tx| {
            tx.query_row(
                "SELECT version, due_at, memory_state FROM target_schedule",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
        })
        .unwrap();
    assert_eq!(version, 1);
    assert_eq!(due, timestamps::isoformat(expected.1));
    assert_eq!(memory, expected.0);
    let summary = learning.readiness.summary(Language::En, None).unwrap();
    assert_eq!((summary.new_count, summary.due_count), (0, 0));
    assert_eq!(summary.next_due_at.as_deref(), Some(due.as_str()));
    assert!(
        learning
            .sessions
            .start(&SessionStart {
                voluntary: false,
                id: Uuid::new_v4(),
                language: Language::En,
                lesson_id: Some(learning.publication.lesson_id),
                dialogue_id: None,
                takeover_session_id: None,
            })
            .is_err()
    );
}

#[test]
fn completed_study_does_not_silently_cover_a_recreated_revision() {
    let learning = Learning::new();
    let study = learning.start(None);
    learning.event(study.id, 1, PracticeKind::Present).unwrap();
    learning.event(study.id, 2, PracticeKind::Next).unwrap();
    let recall = learning.start(None);
    learning.event(recall.id, 1, PracticeKind::Present).unwrap();
    learning.event(recall.id, 2, PracticeKind::Answer).unwrap();
    learning.event(recall.id, 3, PracticeKind::Rate).unwrap();
    assert_eq!(learning.schedule_count(), 1);
    learning.clock.advance(6 * 60);
    let store = Arc::new(MediaStore::open(learning.fixture.directory.path()).unwrap());
    let replacement = publish(&learning.fixture, &learning.clock, &store, true, false);
    assert_eq!(replacement.target_ids, learning.publication.target_ids);
    let next = learning.start(None);
    assert_eq!(next.manifest.targets[0].phrase.revision, 2);
    assert_eq!(next.manifest.targets[0].eligibility, Eligibility::Study);
    let before: (i64, String, String) = learning
        .fixture
        .database
        .read(|tx| {
            tx.query_row(
                "SELECT version, due_at, memory_state FROM target_schedule",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
        })
        .unwrap();
    learning.event(next.id, 1, PracticeKind::Present).unwrap();
    let studied = learning.event(next.id, 2, PracticeKind::Next).unwrap();
    assert!(!studied.receipts[0].schedule_changed);
    assert!(studied.session.attempts[0].rating.is_none());
    let after: (i64, String, String) = learning
        .fixture
        .database
        .read(|tx| {
            tx.query_row(
                "SELECT version, due_at, memory_state FROM target_schedule",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
        })
        .unwrap();
    assert_eq!(after, before);
    let recall = learning.start(None);
    assert_eq!(recall.manifest.targets[0].eligibility, Eligibility::New);
    assert_eq!(recall.manifest.targets[0].schedule_version, before.0);
    learning.event(recall.id, 1, PracticeKind::Present).unwrap();
    learning.event(recall.id, 2, PracticeKind::Answer).unwrap();
    let checked = learning.event(recall.id, 3, PracticeKind::Rate).unwrap();
    assert_eq!(
        checked.receipts[0].eligibility_reason.as_deref(),
        Some("eligible")
    );
    assert_eq!(checked.receipts[0].schedule_version, Some(before.0 + 1));
    let expected = ReviewPolicy::default()
        .review(
            Some(&before.2),
            RecallRating::Recalled,
            learning.clock.now(),
        )
        .unwrap();
    let memory: String = learning
        .fixture
        .database
        .read(|tx| {
            tx.query_row("SELECT memory_state FROM target_schedule", [], |row| {
                row.get(0)
            })
        })
        .unwrap();
    assert_eq!(memory, expected.0);
    let ready = learning.readiness.summary(Language::En, None).unwrap();
    assert_eq!((ready.new_count, ready.due_count), (0, 0));
}
