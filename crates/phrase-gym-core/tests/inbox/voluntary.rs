use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::practice::PracticeKind;
use phrase_gym_contracts::sessions::{Eligibility, SessionStart, SessionStatus};
use uuid::Uuid;

use super::learning::Learning;

fn request(learning: &Learning) -> SessionStart {
    SessionStart {
        voluntary: true,
        id: Uuid::new_v4(),
        language: Language::En,
        lesson_id: Some(learning.publication.lesson_id),
        dialogue_id: None,
        takeover_session_id: None,
    }
}

fn schedules(learning: &Learning) -> Vec<(String, i64, String, Option<String>)> {
    learning.fixture.database.read(|tx| {
        tx.prepare("SELECT target_id, version, due_at, memory_state FROM target_schedule ORDER BY target_id")?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
            .collect()
    }).unwrap()
}

#[test]
fn voluntary_study_retains_genuine_completion_then_ratings_create_no_schedule() {
    let learning = Learning::new();
    let study = learning.sessions.start(&request(&learning)).unwrap();
    assert!(study.manifest.voluntary);
    assert_eq!(study.manifest.targets[0].eligibility, Eligibility::Study);
    learning.event(study.id, 1, PracticeKind::Present).unwrap();
    learning.event(study.id, 2, PracticeKind::Next).unwrap();
    assert_eq!(learning.schedule_count(), 0);
    assert_eq!(
        learning
            .readiness
            .summary(Language::En, None)
            .unwrap()
            .new_count,
        1
    );
    let rehearsal = learning.sessions.start(&request(&learning)).unwrap();
    learning
        .event(rehearsal.id, 1, PracticeKind::Present)
        .unwrap();
    learning
        .event(rehearsal.id, 2, PracticeKind::Explain)
        .unwrap();
    let rating = learning.event(rehearsal.id, 3, PracticeKind::Rate).unwrap();
    assert_eq!(
        rating.receipts[0].eligibility_reason.as_deref(),
        Some("voluntary_practice")
    );
    assert!(!rating.receipts[0].schedule_changed);
    assert!(rating.session.attempts[0].rating.is_some());
    assert_eq!(learning.schedule_count(), 0);
    // Voluntary support still applies; wait for the incidental support window before recall.
    learning.clock.advance(6 * 60);
    let scheduled = learning.start(None);
    learning
        .event(scheduled.id, 1, PracticeKind::Present)
        .unwrap();
    learning
        .event(scheduled.id, 2, PracticeKind::Answer)
        .unwrap();
    assert!(
        learning
            .event(scheduled.id, 3, PracticeKind::Rate)
            .unwrap()
            .receipts[0]
            .schedule_changed
    );
}

#[test]
fn voluntary_ratings_keep_future_and_due_schedules_and_review_receipts_unchanged() {
    let learning = Learning::new();
    let study = learning.start(None);
    learning.event(study.id, 1, PracticeKind::Present).unwrap();
    learning.event(study.id, 2, PracticeKind::Next).unwrap();
    let recall = learning.start(None);
    learning.event(recall.id, 1, PracticeKind::Present).unwrap();
    learning.event(recall.id, 2, PracticeKind::Answer).unwrap();
    learning.event(recall.id, 3, PracticeKind::Rate).unwrap();
    let before = schedules(&learning);
    assert_eq!(
        learning
            .readiness
            .summary(Language::En, None)
            .unwrap()
            .practice_ready_count,
        1
    );
    for days in [0, 30] {
        learning.clock.advance(days * 86400);
        let start = request(&learning);
        let session = learning.sessions.start(&start).unwrap();
        assert_eq!(learning.sessions.start(&start).unwrap(), session);
        learning
            .event(session.id, 1, PracticeKind::Present)
            .unwrap();
        learning
            .event(session.id, 2, PracticeKind::Explain)
            .unwrap();
        let result = learning.event(session.id, 3, PracticeKind::Rate).unwrap();
        assert_eq!(result.session.status, SessionStatus::Completed);
        assert!(!result.receipts[0].schedule_changed);
        assert_eq!(schedules(&learning), before);
    }
    let count: i64 = learning
        .fixture
        .database
        .read(|tx| tx.query_row("SELECT COUNT(*) FROM review_receipt", [], |row| row.get(0)))
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(
        learning
            .readiness
            .summary(Language::En, None)
            .unwrap()
            .due_count,
        1
    );
    let scheduled = learning.start(None);
    assert_eq!(scheduled.manifest.targets[0].eligibility, Eligibility::Due);
}

#[test]
fn voluntary_start_never_replaces_an_unfinished_session_even_with_takeover() {
    let learning = Learning::new();
    let open = learning.start(None);
    let mut start = request(&learning);
    for takeover in [None, Some(open.id)] {
        start.takeover_session_id = takeover;
        assert!(learning.sessions.start(&start).is_err());
        assert_eq!(learning.sessions.current().unwrap(), Some(open.clone()));
    }
    start.language = Language::De;
    assert!(learning.sessions.start(&start).is_err());
}

#[test]
fn voluntary_scope_and_missing_audio_cannot_produce_an_empty_session() {
    let learning = Learning::new();
    let mut start = request(&learning);
    start.lesson_id = Some(Uuid::new_v4());
    assert!(learning.sessions.start(&start).is_err());
    start.lesson_id = None;
    start.language = Language::De;
    assert!(learning.sessions.start(&start).is_err());
    for entry in std::fs::read_dir(learning.fixture.directory.path().join("media")).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    let summary = learning.readiness.summary(Language::En, None).unwrap();
    assert_eq!(summary.practice_ready_count, 0);
    assert_eq!(summary.preparing_count, 1);
    assert!(learning.sessions.start(&request(&learning)).is_err());
    assert!(learning.sessions.current().unwrap().is_none());
}
