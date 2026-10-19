use std::sync::Arc;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::profiles::{ProfileRestore, ProfileUpdate};
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::error::CoreError;
use phrase_gym_core::profiles::Profiles;
use phrase_gym_core::storage::Database;
use uuid::Uuid;

#[test]
fn revisions_validate_atomically_recover_receipts_and_restore_without_rewriting_history() {
    let directory = tempfile::tempdir().unwrap();
    let database = Arc::new(Database::open(directory.path()).unwrap());
    let profiles = Profiles::new(database, Arc::new(SystemClock));
    profiles.initialize_current_learner().unwrap();
    let initial = profiles.get(None).unwrap();
    assert_eq!(initial.revision, 1);
    assert_eq!(initial.profile.native_language, Language::Ru);
    assert_eq!(initial.profile.learning_timezone, "Europe/Belgrade");
    assert_eq!(
        initial.profile.languages[&Language::En]
            .proficiency
            .as_deref(),
        Some("B2")
    );
    assert_eq!(
        initial.profile.languages[&Language::En].translation_with_answer,
        Some(false)
    );
    assert_eq!(
        initial.profile.languages[&Language::De].help_language,
        Some(Language::Ru)
    );
    assert_eq!(initial.profile.languages[&Language::Sr].help_language, None);
    assert_eq!(
        initial.profile.languages[&Language::Sr].translation_with_answer,
        None
    );
    let mut update = ProfileUpdate {
        mutation_id: Uuid::new_v4(),
        expected_revision: 1,
        profile: initial.profile.clone(),
        reason: "Learner requested timezone change".into(),
    };
    update.profile.learning_timezone = "not/a/timezone".into();
    assert!(matches!(
        profiles.update("preferences-agent", &update),
        Err(CoreError::Invalid(_))
    ));
    assert_eq!(profiles.get(None).unwrap(), initial);
    update.profile.learning_timezone = "UTC".into();
    let second = profiles.update("preferences-agent", &update).unwrap();
    assert_eq!(second.revision, 2);
    assert_eq!(second.actor, "preferences-agent");
    assert_eq!(
        profiles.update("preferences-agent", &update).unwrap(),
        second
    );
    assert!(matches!(
        profiles.update("other-agent", &update),
        Err(CoreError::Conflict(_))
    ));
    let mut invalid_retry = update.clone();
    invalid_retry.profile.learning_timezone = "not/a/timezone".into();
    assert!(matches!(
        profiles.update("preferences-agent", &invalid_retry),
        Err(CoreError::Conflict(_))
    ));
    let mut changed = update.clone();
    changed.reason = "Conflicting retry".into();
    assert!(matches!(
        profiles.update("preferences-agent", &changed),
        Err(CoreError::Conflict(_))
    ));
    changed.mutation_id = Uuid::new_v4();
    assert!(matches!(
        profiles.update("preferences-agent", &changed),
        Err(CoreError::Conflict(_))
    ));
    let restore = ProfileRestore {
        mutation_id: Uuid::new_v4(),
        expected_revision: 2,
        restore_revision: 1,
        reason: "Restore learner preference".into(),
    };
    let third = profiles.restore("preferences-agent", &restore).unwrap();
    assert_eq!(third.revision, 3);
    assert_eq!(third.restored_from, Some(1));
    assert_eq!(third.profile, initial.profile);
    assert_eq!(profiles.get(Some(2)).unwrap(), second);
    assert_eq!(
        profiles.update("preferences-agent", &update).unwrap(),
        second
    );
    assert_eq!(
        profiles.restore("preferences-agent", &restore).unwrap(),
        third
    );
    profiles.initialize_current_learner().unwrap();
    assert_eq!(profiles.get(None).unwrap(), third);
    let reopened = Profiles::new(
        Arc::new(Database::open(directory.path()).unwrap()),
        Arc::new(SystemClock),
    );
    assert_eq!(reopened.get(None).unwrap(), third);
}

#[test]
fn committed_update_and_restore_receipts_precede_reason_validation() {
    let directory = tempfile::tempdir().unwrap();
    let profiles = Profiles::new(
        Arc::new(Database::open(directory.path()).unwrap()),
        Arc::new(SystemClock),
    );
    profiles.initialize_current_learner().unwrap();
    let update = ProfileUpdate {
        mutation_id: Uuid::new_v4(),
        expected_revision: 1,
        profile: profiles.get(None).unwrap().profile,
        reason: "Requested change".into(),
    };
    let second = profiles.update("agent", &update).unwrap();
    let restore = ProfileRestore {
        mutation_id: Uuid::new_v4(),
        expected_revision: 2,
        restore_revision: 1,
        reason: "Requested restore".into(),
    };
    let third = profiles.restore("agent", &restore).unwrap();
    for reason in [String::new(), " ".into(), "a".repeat(2_001)] {
        let mut retry = update.clone();
        retry.reason.clone_from(&reason);
        assert!(matches!(
            profiles.update("agent", &retry),
            Err(CoreError::Conflict(_))
        ));
        retry.mutation_id = Uuid::new_v4();
        retry.expected_revision = third.revision;
        assert!(matches!(
            profiles.update("agent", &retry),
            Err(CoreError::Invalid(_))
        ));
        let mut retry = restore.clone();
        retry.reason = reason;
        assert!(matches!(
            profiles.restore("agent", &retry),
            Err(CoreError::Conflict(_))
        ));
        retry.mutation_id = Uuid::new_v4();
        retry.expected_revision = third.revision;
        assert!(matches!(
            profiles.restore("agent", &retry),
            Err(CoreError::Invalid(_))
        ));
        assert_eq!(profiles.get(None).unwrap(), third);
    }
    assert_eq!(profiles.update("agent", &update).unwrap(), second);
    assert_eq!(profiles.restore("agent", &restore).unwrap(), third);
}

#[test]
fn concurrent_expected_revision_writes_have_one_winner() {
    let directory = tempfile::tempdir().unwrap();
    let profiles = Arc::new(Profiles::new(
        Arc::new(Database::open(directory.path()).unwrap()),
        Arc::new(SystemClock),
    ));
    profiles.initialize_current_learner().unwrap();
    let profile = profiles.get(None).unwrap().profile;
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let handles = (0..2)
        .map(|_| {
            let profiles = profiles.clone();
            let barrier = barrier.clone();
            let profile = profile.clone();
            std::thread::spawn(move || {
                barrier.wait();
                profiles.update(
                    "agent",
                    &ProfileUpdate {
                        mutation_id: Uuid::new_v4(),
                        expected_revision: 1,
                        profile,
                        reason: "Concurrent request".into(),
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(CoreError::Conflict(_))))
            .count(),
        1
    );
    assert_eq!(profiles.get(None).unwrap().revision, 2);
}
