use std::sync::Arc;

use phrase_gym_contracts::practice::{Modality, PracticeKind};
use phrase_gym_core::clock::Clock;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::sessions::Sessions;
use phrase_gym_core::storage::Database;
use phrase_gym_core::timestamps;

use super::learning::Learning;

#[test]
fn independent_situations_rotate_after_due_reviews_and_restart_without_changing_pinned_attempts() {
    let mut learning = Learning::varied();
    let study = learning.start(None);
    learning.event(study.id, 1, PracticeKind::Present).unwrap();
    learning.event(study.id, 2, PracticeKind::Next).unwrap();
    let studied = learning.sessions.get(study.id).unwrap();
    // Study presentation does not consume the first independent situation.
    for (index, expected) in ["call", "room", "call"].into_iter().enumerate() {
        let recall = learning.start(None);
        assert_eq!(recall.manifest.targets[0].cue_id, expected);
        learning.event(recall.id, 1, PracticeKind::Present).unwrap();
        learning.event(recall.id, 2, PracticeKind::Answer).unwrap();
        let rated = learning.event(recall.id, 3, PracticeKind::Rate).unwrap();
        assert_eq!(
            rated.receipts[0].eligibility_reason.as_deref(),
            Some("eligible")
        );
        assert!(rated.receipts[0].schedule_changed);
        assert_eq!(
            rated.receipts[0].schedule_version,
            Some(i64::try_from(index).unwrap() + 1)
        );
        let due = timestamps::parse(rated.receipts[0].due_at.as_ref().unwrap()).unwrap();
        learning
            .clock
            .advance((due - learning.clock.now()).num_seconds() + 1);
        learning.sessions = Sessions::new(
            Arc::new(Database::open(learning.fixture.directory.path()).unwrap()),
            Arc::new(MediaStore::open(learning.fixture.directory.path()).unwrap()),
            learning.clock.clone(),
        );
        let retained = learning.sessions.get(recall.id).unwrap();
        assert_eq!(retained.manifest, recall.manifest);
        assert_eq!(retained.attempts[0].cue_id, expected);
        assert_eq!(
            retained.attempts[0].original.as_ref().unwrap().modality,
            Modality::SelfReport
        );
        assert_eq!(
            retained.attempts[0].revision,
            recall.manifest.targets[0].phrase.revision
        );
        assert_eq!(learning.sessions.get(study.id).unwrap(), studied);
    }
    let reported = learning.start(None);
    let cue = reported.manifest.targets[0].cue_id.clone();
    learning
        .event(reported.id, 1, PracticeKind::Present)
        .unwrap();
    learning
        .event(reported.id, 2, PracticeKind::Report)
        .unwrap();
    let next = learning.start(None);
    assert_ne!(next.manifest.targets[0].cue_id, cue);
}

#[test]
fn word_contexts_rotate_matching_answers_and_references_on_one_schedule_across_due_return() {
    let learning = Learning::contextual_word();
    let study = learning.start(None);
    learning.event(study.id, 1, PracticeKind::Present).unwrap();
    learning.event(study.id, 2, PracticeKind::Next).unwrap();
    let target = study.manifest.targets[0].phrase.id;
    for expected in ["context-0", "context-1", "context-2", "context-0"] {
        let recall = learning.start(None);
        let item = &recall.manifest.targets[0];
        assert_eq!(item.phrase.id, target);
        assert_eq!(item.cue_id, expected);
        let phrase_gym_contracts::english::EnglishTask::WordSense { contexts, .. } =
            item.phrase.english.as_ref().unwrap().task.as_ref().unwrap()
        else {
            panic!("Expected word task")
        };
        let context = contexts
            .iter()
            .find(|context| context.situation_id == expected)
            .unwrap();
        assert_eq!(item.phrase.content.text, context.answer);
        assert_eq!(item.phrase.content.chunks[0].text, context.answer);
        assert_eq!(
            item.phrase.content.reference_audio_id,
            Some(context.reference_audio_id)
        );
        assert!(
            recall
                .manifest
                .required_media_ids
                .contains(&context.reference_audio_id)
        );
        learning.event(recall.id, 1, PracticeKind::Present).unwrap();
        learning.event(recall.id, 2, PracticeKind::Answer).unwrap();
        let rated = learning.event(recall.id, 3, PracticeKind::Rate).unwrap();
        assert!(rated.receipts[0].schedule_changed);
        assert_eq!(learning.schedule_count(), 1);
        let due = timestamps::parse(rated.receipts[0].due_at.as_ref().unwrap()).unwrap();
        learning
            .clock
            .advance((due - learning.clock.now()).num_seconds() + 1);
    }
}
