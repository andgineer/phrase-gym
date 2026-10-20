use std::sync::Arc;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::inbox::PublicationReceipt;
use phrase_gym_contracts::practice::{
    EventBatch, Modality, PracticeInput, PracticeKind, RecallRating,
};
use phrase_gym_contracts::sessions::{PracticeBatchReceipt, SessionStart};
use phrase_gym_core::clock::Clock;
use phrase_gym_core::english::EnglishPackages;
use phrase_gym_core::inbox::{Inbox, delivery::Delivery, jobs::Jobs};
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::practice::Practice;
use phrase_gym_core::readiness::Readiness;
use phrase_gym_core::sessions::Sessions;
use phrase_gym_core::timestamps;
use uuid::Uuid;

use super::{TestClock, input, output, support::Fixture};

pub(super) struct Learning {
    pub(super) fixture: Fixture,
    pub(super) clock: Arc<TestClock>,
    pub(super) sessions: Sessions,
    practice: Practice,
    pub(super) readiness: Readiness,
    pub(super) publication: PublicationReceipt,
}

impl Learning {
    pub(super) fn new() -> Self {
        Self::create(false)
    }

    pub(super) fn varied() -> Self {
        Self::create(true)
    }

    fn create(varied: bool) -> Self {
        let fixture = Fixture::new();
        let clock = Arc::new(TestClock::new());
        let store = Arc::new(MediaStore::open(fixture.directory.path()).unwrap());
        let publication = publish(&fixture, &clock, &store, false, varied);
        Self {
            sessions: Sessions::new(fixture.database.clone(), store.clone(), clock.clone()),
            practice: Practice::new(fixture.database.clone(), clock.clone()),
            readiness: Readiness::new(fixture.database.clone(), store, clock.clone()),
            fixture,
            clock,
            publication,
        }
    }

    pub(super) fn start(
        &self,
        takeover: Option<Uuid>,
    ) -> phrase_gym_contracts::sessions::PracticeSession {
        self.sessions
            .start(&SessionStart {
                id: Uuid::new_v4(),
                language: Language::En,
                lesson_id: Some(self.publication.lesson_id),
                dialogue_id: None,
                takeover_session_id: takeover,
            })
            .unwrap()
    }

    pub(super) fn event(
        &self,
        session: Uuid,
        sequence: i64,
        kind: PracticeKind,
    ) -> Result<PracticeBatchReceipt, phrase_gym_core::error::CoreError> {
        self.practice.submit(
            session,
            &EventBatch {
                events: vec![PracticeInput {
                    id: Uuid::new_v4(),
                    sequence,
                    step: 0,
                    occurred_at: timestamps::isoformat(self.clock.now()),
                    kind,
                    modality: Modality::SelfReport,
                    rating: (kind == PracticeKind::Rate).then_some(RecallRating::Recalled),
                    reason: (kind == PracticeKind::Report).then(|| "Ambiguous cue".into()),
                }],
            },
        )
    }

    pub(super) fn schedule_count(&self) -> i64 {
        self.fixture
            .database
            .read(|tx| tx.query_row("SELECT COUNT(*) FROM target_schedule", [], |row| row.get(0)))
            .unwrap()
    }
}

pub(super) fn publish(
    f: &Fixture,
    clock: &Arc<TestClock>,
    store: &Arc<MediaStore>,
    replacement: bool,
    varied: bool,
) -> PublicationReceipt {
    let inbox = Inbox::new(f.database.clone(), store.clone(), clock.clone());
    let mut submission = input(f);
    if replacement {
        submission.expected_generation = Some(1);
        submission.phrases[0].expected_revision = Some(1);
    }
    inbox.submit("sender", &submission).unwrap();
    let jobs = Jobs::new(f.database.clone(), clock.clone());
    let claim = jobs.claim("worker", Uuid::new_v4()).unwrap().unwrap();
    let mut lesson = output(f, &claim);
    if varied {
        let mut second = lesson.package.phrases[0].situations[0].clone();
        second.id = "room".into();
        second.text = "The group is ready. Begin the workshop.".into();
        lesson.package.phrases[0].situations.push(second);
    }
    if replacement {
        lesson.package.phrases[0].expected_revision = Some(1);
    }
    let delivery = Delivery::new(
        f.database.clone(),
        Arc::new(EnglishPackages::new(
            f.database.clone(),
            store.clone(),
            f.profiles.clone(),
        )),
        clock.clone(),
    );
    delivery.stage("worker", &lesson).unwrap();
    delivery.publish("worker", &claim.fence).unwrap()
}
