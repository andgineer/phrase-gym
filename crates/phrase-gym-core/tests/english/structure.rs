use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::english::{PartnerPrompt, PhraseFamiliarity, PreparedEnglishLesson};
use serde_json::json;

use super::support::{ANSWER, Fixture};

type Mutation = (&'static str, fn(&mut PreparedEnglishLesson));

#[test]
fn the_bounded_english_contract_rejects_unsupported_material_and_structural_leakage() {
    let fixture = Fixture::new();
    let package = fixture.package();
    assert!(fixture.validator.validate(&package).is_ok());
    let cases: Vec<Mutation> = vec![
        ("non-English", |package| package.language = Language::De),
        ("no phrases", |package| package.phrases.clear()),
        ("too many", |package| {
            package.phrases = vec![package.phrases[0].clone(); 6];
        }),
        ("membership", |package| {
            package.lesson.target_source_ids.clear();
        }),
        ("profile", |package| package.profile_revision = 0),
        ("guide", |package| package.guide_version.clear()),
        ("assessment", |package| package.assessment_version.clear()),
        ("no situations", |package| {
            package.phrases[0].situations.clear();
        }),
        ("too many situations", |package| {
            package.phrases[0].situations = vec![package.phrases[0].situations[0].clone(); 3];
        }),
        ("lesson title", |package| {
            package.lesson.title = ANSWER.into();
        }),
        ("full answer", |package| {
            package.phrases[0].situations[0].text = format!("Say {ANSWER}");
        }),
        ("normalized answer", |package| {
            package.phrases[0].situations[0].text = "LET S GET STARTED!".into();
        }),
        ("accepted answer", |package| {
            package.phrases[0].situations[0].text = "Ask: Shall we begin?".into();
        }),
        ("gapped cue", |package| {
            package.phrases[0].situations[0].text = "Let's ___ started.".into();
        }),
        ("partner leak", |package| {
            package.phrases[0].situations[0].partner = Some(PartnerPrompt {
                text: ANSWER.into(),
                audio_id: None,
            });
        }),
        ("hint reveals answer", |package| {
            package.phrases[0].hint = ANSWER.into();
        }),
        ("hint is unrelated", |package| {
            package.phrases[0].hint = "unrelated".into();
        }),
        ("wrong help language", |package| {
            package.phrases[0].help.as_mut().unwrap().language = Language::Ru;
        }),
        ("translation off", |package| {
            package.phrases[0].help.as_mut().unwrap().translation = Some("Начнём".into());
        }),
        ("unknown exposure", |package| {
            package.phrases[0].situations[0]
                .exposed_target_source_ids
                .push("missing".into());
        }),
        ("self exposure", |package| {
            package.phrases[0].situations[0]
                .exposed_target_source_ids
                .push("open-meeting".into());
        }),
        ("unproven studied", |package| {
            package.phrases[0].familiarity = PhraseFamiliarity::Studied {
                provenance: String::new(),
            };
        }),
    ];
    for (name, mutate) in cases {
        let mut changed = package.clone();
        mutate(&mut changed);
        assert!(fixture.validator.validate(&changed).is_err(), "{name}");
    }
    let mut value = serde_json::to_value(&package).unwrap();
    value["phrases"][0]["cues"] = json!([]);
    assert!(serde_json::from_value::<PreparedEnglishLesson>(value).is_err());
    let mut value = serde_json::to_value(&package).unwrap();
    value["schema_version"] = json!(1);
    assert!(serde_json::from_value::<PreparedEnglishLesson>(value).is_err());
}

#[test]
fn hints_must_not_reveal_any_complete_accepted_answer() {
    let fixture = Fixture::new();
    let mut package = fixture.package();
    let phrase = &mut package.phrases[0];
    phrase.text = "Thank you very much.".into();
    phrase.reference_audio_id = fixture.upload(&json!({ "text": phrase.text })).id;
    phrase.accepted_variants = vec!["Thank you.".into()];
    phrase.hint = "Thank".into();
    assert!(fixture.validator.validate(&package).is_ok());
    for hint in ["Thank you.", "THANK, YOU", "Thank you very"] {
        package.phrases[0].hint = hint.into();
        assert!(fixture.validator.validate(&package).is_err(), "{hint}");
    }
}

#[test]
fn profile_revision_is_pinned_and_unresolved_serbian_does_not_block_english() {
    let fixture = Fixture::new();
    let package = fixture.package();
    let mut profile = fixture.profiles.get(None).unwrap().profile;
    profile
        .languages
        .get_mut(&Language::En)
        .unwrap()
        .translation_with_answer = Some(true);
    fixture
        .profiles
        .update(
            "agent",
            &phrase_gym_contracts::profiles::ProfileUpdate {
                mutation_id: uuid::Uuid::new_v4(),
                expected_revision: 1,
                profile,
                reason: "Learner requests translation".into(),
            },
        )
        .unwrap();
    assert!(fixture.validator.validate(&package).is_ok());
    let mut changed = package.clone();
    changed.profile_revision = 2;
    assert!(fixture.validator.validate(&changed).is_err());
    changed.phrases[0].help.as_mut().unwrap().translation = Some("Начнём".into());
    assert!(fixture.validator.validate(&changed).is_ok());
    changed.profile_revision = 999;
    assert!(fixture.validator.validate(&changed).is_err());
}
