use std::sync::Arc;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::sessions::{Eligibility, SessionStart};
use phrase_gym_core::english::EnglishPackages;
use phrase_gym_core::inbox::{Inbox, delivery::Delivery, jobs::Jobs};
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::readiness::Readiness;
use phrase_gym_core::sessions::Sessions;
use uuid::Uuid;

use super::{TestClock, input, output, support::Fixture};

// Six separate lessons, each with one English target that still needs its first study.
fn six_unstudied(f: &Fixture, clock: &Arc<TestClock>, store: &Arc<MediaStore>) {
    let inbox = Inbox::new(f.database.clone(), store.clone(), clock.clone());
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let packages = EnglishPackages::new(f.database.clone(), store.clone(), f.profiles.clone());
    let delivery = Delivery::new(f.database.clone(), Arc::new(packages), clock.clone());
    for index in 0..6 {
        let mut submission = input(f);
        submission.item_id = format!("meeting-{index}");
        submission.phrases[0].source_id = format!("open-{index}");
        inbox.submit("sender", &submission).unwrap();
        let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
        let mut lesson = output(f, &claim);
        lesson
            .package
            .lesson
            .source_id
            .clone_from(&submission.item_id);
        lesson.package.lesson.target_source_ids = vec![submission.phrases[0].source_id.clone()];
        let phrase = &mut lesson.package.phrases[0];
        phrase
            .source_id
            .clone_from(&submission.phrases[0].source_id);
        phrase.reference_audio_id = submission.phrases[0].reference_audio_id.unwrap();
        delivery.stage("worker", &lesson).unwrap();
        delivery.publish("worker", &claim.fence).unwrap();
    }
}

fn start(sessions: &Sessions, voluntary: bool) -> phrase_gym_contracts::sessions::PracticeSession {
    sessions
        .start(&SessionStart {
            voluntary,
            id: Uuid::new_v4(),
            language: Language::En,
            lesson_id: None,
            dialogue_id: None,
            takeover_session_id: None,
        })
        .unwrap()
}

fn library() -> (Fixture, Sessions, Readiness) {
    let fixture = Fixture::new();
    let clock = Arc::new(TestClock::new());
    let store = Arc::new(MediaStore::open(fixture.directory.path()).unwrap());
    six_unstudied(&fixture, &clock, &store);
    let readiness = Readiness::new(fixture.database.clone(), store.clone(), clock.clone());
    let sessions = Sessions::new(fixture.database.clone(), store, clock);
    (fixture, sessions, readiness)
}

#[test]
fn first_study_counts_as_new_and_a_scheduled_round_introduces_at_most_five() {
    let (_fixture, sessions, readiness) = library();
    let summary = readiness.summary(Language::En, None).unwrap();
    assert_eq!(
        (
            summary.new_count,
            summary.due_count,
            summary.practice_ready_count
        ),
        (6, 0, 6)
    );
    let scheduled = start(&sessions, false);
    assert_eq!(scheduled.manifest.targets.len(), 5);
    assert!(
        scheduled
            .manifest
            .targets
            .iter()
            .all(|target| target.eligibility == Eligibility::Study)
    );
}

#[test]
fn a_voluntary_round_never_introduces_more_than_five_first_meetings() {
    let (_fixture, sessions, _) = library();
    let voluntary = start(&sessions, true);
    assert!(voluntary.manifest.voluntary);
    assert_eq!(voluntary.manifest.targets.len(), 5);
}
