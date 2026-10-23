use super::*;
use serde_json::json;

#[test]
fn truncated_word_references_fail_before_speech_but_explicit_cues_are_valid() {
    let mut phrase: PreparedEnglishPhrase = serde_json::from_value(json!({
            "source_id":"one", "meaning_key":"clarify", "intention_key":"clarify", "text":"Please clarify the deadline.", "accepted_variants":[],
            "situations":[{"id":"a", "text":"Please _____ the deadline. | Make it clear.", "language":"en", "audio_id":null, "partner":null, "exposed_target_source_ids":[]}, {"id":"b", "text":"Can you _____ this point?", "language":"en", "audio_id":null, "partner":null, "exposed_target_source_ids":[]}],
            "hint":"cl", "help":null, "familiarity":{"kind":"new"}, "reference_audio_id":Uuid::nil(), "expected_revision":null,
            "task":{"kind":"word_sense", "word":"clarify", "sense":"make clear", "contexts":[{"situation_id":"a", "answer":"Please clarify the deadline.", "reference_audio_id":Uuid::nil()}, {"situation_id":"b", "answer":"Can you clarify this point?", "reference_audio_id":Uuid::nil()}]}
        })).unwrap();
    validate_task_shape(&phrase).unwrap();
    for truncated in ["Can you clarify", "Can you clar"] {
        let Some(EnglishTask::WordSense { contexts, .. }) = &mut phrase.task else {
            unreachable!()
        };
        contexts[1].answer = truncated.into();
        assert!(
            validate_task_shape(&phrase)
                .unwrap_err()
                .contains("nothing was synthesized")
        );
    }
}

#[test]
fn punctuation_only_variants_fail_before_speech_without_rewriting_model_output() {
    let mut phrase: PreparedEnglishPhrase = serde_json::from_value(json!({
            "source_id":"one", "meaning_key":"repeat", "intention_key":"repeat", "text":"Could you repeat that, please?", "accepted_variants":["Could you repeat that please?"],
            "situations":[], "hint":"repeat", "help":null, "familiarity":{"kind":"new"}, "reference_audio_id":Uuid::nil(), "expected_revision":null
        })).unwrap();
    assert!(
        validate_variants(&phrase)
            .unwrap_err()
            .contains("nothing was synthesized")
    );
    assert_eq!(phrase.accepted_variants, ["Could you repeat that please?"]);
    phrase.accepted_variants = vec!["Could you say that again, please?".into()];
    validate_variants(&phrase).unwrap();
}

#[test]
fn a_subscription_usage_limit_stops_as_quota_rather_than_a_generic_failure() {
    let attempt = tempfile::tempdir().unwrap();
    std::fs::write(attempt.path().join("stdout.log"), b"").unwrap();
    std::fs::write(
        attempt.path().join("stderr.log"),
        "ERROR: You've hit your usage limit. Try again later.",
    )
    .unwrap();
    assert!(failure(attempt.path(), Ending::Exited(Some(1))).contains("quota"));
    std::fs::write(attempt.path().join("stderr.log"), "unexpected failure").unwrap();
    assert!(!failure(attempt.path(), Ending::Exited(Some(1))).contains("quota"));
}

#[test]
fn reordered_output_preserves_valid_cross_target_exposures_and_rejects_self_or_foreign_before_speech()
 {
    let generation = Uuid::new_v4();
    let claim: JobClaim = serde_json::from_value(json!({
        "fence":{"generation_id":generation,"claim_id":Uuid::new_v4(),"token":Uuid::new_v4(),"fence":1},
        "lease_until":"2026-10-23T12:00:00Z","deadline":"2026-10-23T12:08:00Z",
        "receipt":{"generation_id":generation,"submission_id":Uuid::new_v4(),"generation":1,"namespace":"synthetic","profile_revision":1,"guide_version":"test","received_at":"2026-10-23T11:58:00Z"},
        "input":{"submission_id":Uuid::new_v4(),"item_id":"batch","title":"Synthetic","language":"en","context":"synthetic","note":"","expected_generation":null,
            "phrases":[{"source_id":"one","text":"Let's get started.","reference_audio_id":null,"audio_sha256":null,"expected_revision":null},{"source_id":"other","text":"Let's get started.","reference_audio_id":null,"audio_sha256":null,"expected_revision":null}]},
        "profile":{"revision":1,"profile":{"native_language":"ru","learning_timezone":"Europe/Belgrade","languages":{}},"mutation_id":null,"actor":"synthetic","reason":"test","created_at":"2026-10-23T11:00:00Z","restored_from":null},"guide":"Synthetic guide"
    })).unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/coach/supplied-lesson.json"
    ))
    .unwrap();
    let mut first = fixture["phrase"].clone();
    first["source_id"] = json!("one");
    let mut second = fixture["phrase"].clone();
    second["source_id"] = json!("other");
    let directory = tempfile::tempdir().unwrap();
    for (source, valid) in [("other", true), ("one", false), ("unknown", false)] {
        first["situations"][0]["exposed_target_source_ids"] = json!([source]);
        let output = json!({"phrases":[second, first]});
        std::fs::write(directory.path().join("stdout.log"), output.to_string()).unwrap();
        let result = read_selected(directory.path(), &claim, "synthetic");
        if valid {
            let phrases = result.unwrap().package.phrases;
            assert_eq!(
                phrases
                    .iter()
                    .map(|phrase| phrase.source_id.as_str())
                    .collect::<Vec<_>>(),
                ["one", "other"]
            );
            assert_eq!(phrases[0].situations[0].exposed_target_source_ids, [source]);
        } else {
            assert!(
                result
                    .unwrap_err()
                    .contains("nothing was synthesized or published")
            );
        }
        let retained: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.path().join("stdout.log")).unwrap())
                .unwrap();
        assert_eq!(
            retained, output,
            "Never repair model-authored exposures silently"
        );
    }
}

#[test]
fn exact_model_frame_with_a_spaced_slot_name_fails_before_speech_without_repair() {
    let mut phrase: PreparedEnglishPhrase = serde_json::from_value(json!({
        "source_id":"item-1", "meaning_key":"wait", "intention_key":"wait", "source_text":"Could you give me a moment?", "text":"Could you give me a moment?", "accepted_variants":[], "situations":[], "hint":"Could", "help":null, "familiarity":{"kind":"new"}, "reference_audio_id":Uuid::nil(), "expected_revision":null,
        "task":{"kind":"frame", "pattern":"Could you give me {short wait period}?", "slots":[{"name":"short wait period", "values":["a moment", "a minute", "a second"]}]}
    })).unwrap();
    let unchanged = phrase.clone();
    assert!(
        validate_task_shape(&phrase)
            .unwrap_err()
            .contains("nothing was synthesized or published")
    );
    assert_eq!(phrase, unchanged);
    for name in ["short-wait-period", "время", "", &"a".repeat(41)] {
        let Some(EnglishTask::Frame { pattern, slots }) = &mut phrase.task else {
            unreachable!()
        };
        *pattern = format!("Could you give me {{{name}}}?");
        slots[0].name = name.into();
        assert_eq!(
            validate_task_shape(&phrase).is_ok(),
            matches!(name, "short-wait-period" | "время"),
            "{name}"
        );
    }
}
