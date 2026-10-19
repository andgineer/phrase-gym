use phrase_gym_core::english::target_id;
use phrase_gym_core::error::CoreError;
use rusqlite::params;

use super::support::{ANSWER, Fixture};

#[test]
fn identity_is_stable_across_grouping_and_editorial_changes_but_meanings_cannot_be_reused() {
    let fixture = Fixture::new();
    let package = fixture.package();
    let ids = fixture.validator.validate(&package).unwrap();
    let id = ids["open-meeting"];
    assert_eq!(id, target_id(&package.namespace, "open-meeting"));
    let mut changed = package.clone();
    changed.lesson.title = "Changed title".into();
    changed.source_revision = "2".into();
    changed.guide_version = "fixture-guide-v2".into();
    assert_eq!(fixture.validator.validate(&changed).unwrap(), ids);
    assert_ne!(target_id("other", "open-meeting"), id);
    fixture.database.write(|transaction| {
        transaction.execute("INSERT INTO target (id, language, kind, meaning_key, recall_expression, current_revision) VALUES (?1, 'en', 'fixed_phrase', 'begin-meeting', ?2, 1)", params![id.to_string(), ANSWER])?;
        transaction.execute("INSERT INTO source_item (namespace, source_id, target_id, source_revision) VALUES (?1, 'open-meeting', ?2, '1')", params![package.namespace, id.to_string()])
    }).unwrap();
    assert!(matches!(
        fixture.validator.validate(&package),
        Err(CoreError::Conflict(_))
    ));
    changed.phrases[0].expected_revision = Some(1);
    assert_eq!(fixture.validator.validate(&changed).unwrap(), ids);
    changed.phrases[0].meaning_key = "a-new-meaning".into();
    assert!(matches!(
        fixture.validator.validate(&changed),
        Err(CoreError::Invalid(_))
    ));
    changed.phrases[0].source_id = "new-meaning".into();
    changed.phrases[0].expected_revision = None;
    changed.lesson.target_source_ids = vec!["new-meaning".into()];
    assert_ne!(
        fixture.validator.validate(&changed).unwrap()["new-meaning"],
        id
    );
}
