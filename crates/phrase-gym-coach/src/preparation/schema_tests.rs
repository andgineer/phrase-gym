use super::*;
use phrase_gym_contracts::base::Language;

#[test]
fn a_plain_word_requires_contextual_word_retrieval_and_a_partial_hint() {
    let schema = phrase_schema("clarify").unwrap();
    assert_eq!(
        schema["properties"]["task"]["properties"]["kind"]["enum"],
        json!(["word_sense"])
    );
    assert_eq!(
        schema["properties"]["hint"]["enum"],
        json!(["c", "cl", "cla"])
    );
    assert_eq!(
        schema["properties"]["situations"]["items"]["properties"]["text"]["pattern"],
        WORD_SITUATION
    );
    assert_eq!(
        schema["properties"]["situations"]["items"]["properties"]["partner"]["type"],
        "null"
    );
    let frame = phrase_schema("Could we move {item} to {day}?").unwrap();
    assert_eq!(
        frame["properties"]["task"]["properties"]["kind"]["enum"],
        json!(["frame"])
    );
}

#[test]
fn bounded_schema_includes_phrase_dependencies_without_learner_or_access_contracts() {
    let schema = phrase_schema("Let's get started.").unwrap();
    assert!(
        schema["$defs"]["PreparedEnglishPhrase"]["properties"]["reference_audio_id"].is_object()
    );
    assert!(schema["$defs"]["PhraseSituation"].is_object());
    assert!(schema["$defs"]["PhraseHelp"].is_object());
    assert!(schema["$defs"]["RecordingUpload"].is_null());
    assert_eq!(schema["type"], "object");
    assert_eq!(
        schema["properties"]["hint"]["enum"],
        json!(["Let's", "get", "started"])
    );
    assert!(!schema.to_string().contains("oneOf"));
    assert_eq!(
        schema["$defs"]["PhraseHelp"]["required"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert!(schema["$defs"]["PhraseDetails"].is_object());
    assert_eq!(
        schema["$defs"]["PhraseFamiliarity"]["anyOf"][0]["additionalProperties"],
        false
    );
    assert!(!schema.to_string().contains("#/components/"));
}

#[test]
fn exposure_schema_allows_only_other_accepted_sources_and_none_for_single_items() {
    let input = |source: &str| {
        serde_json::from_value(json!({"source_id":source,"text":"Let's get started.","reference_audio_id":null,"audio_sha256":null,"expected_revision":null})).unwrap()
    };
    let schema = batch_schema(&[input("one"), input("two")]).unwrap();
    for (index, other) in [(0, "two"), (1, "one")] {
        let exposures = &schema["properties"]["phrases"]["items"]["anyOf"][index]["properties"]["situations"]
            ["items"]["properties"]["exposed_target_source_ids"];
        assert_eq!(exposures["items"]["enum"], json!([other]));
        assert_eq!(exposures["maxItems"], 1);
    }
    let single = batch_schema(&[input("one")]).unwrap();
    assert_eq!(
        single["properties"]["situations"]["items"]["properties"]["exposed_target_source_ids"]["maxItems"],
        0
    );
}

#[test]
fn generated_frame_slot_names_use_bounded_ascii_identifiers_for_plain_or_declared_frames() {
    for input in [
        "Could you send me the report?",
        "Could you send me {shared-document}?",
    ] {
        let schema = phrase_schema(input).unwrap();
        let name = &schema["$defs"]["FrameSlot"]["properties"]["name"];
        assert_eq!(name["pattern"], "^[A-Za-z0-9-]+$");
        assert_eq!(name["minLength"], 1);
        assert_eq!(name["maxLength"], english_source::FRAME_SLOT_NAME_MAX_BYTES);
    }
}

#[test]
fn both_help_layers_are_required_in_their_pinned_languages() {
    let input = |source: &str| {
        serde_json::from_value(json!({"source_id":source,"text":"Let's get started.","reference_audio_id":null,"audio_sha256":null,"expected_revision":null})).unwrap()
    };
    let mut schema = batch_schema(&[input("one"), input("two")]).unwrap();
    pin_help_languages(&mut schema, Language::En, Language::Ru);
    assert_eq!(
        schema["$defs"]["PhraseHelp"]["properties"]["language"]["enum"],
        json!(["en"])
    );
    assert_eq!(
        schema["$defs"]["PhraseDetails"]["properties"]["language"]["enum"],
        json!(["ru"])
    );
    for item in schema["properties"]["phrases"]["items"]["anyOf"]
        .as_array()
        .unwrap()
    {
        assert_eq!(
            item["properties"]["help"],
            json!({"$ref": "#/$defs/PhraseHelp"})
        );
        assert_eq!(
            item["properties"]["details"],
            json!({"$ref": "#/$defs/PhraseDetails"})
        );
        let required = item["required"].as_array().unwrap();
        assert!(required.contains(&json!("help")) && required.contains(&json!("details")));
    }
    let details = &schema["$defs"]["PhraseDetails"];
    assert_eq!(details["additionalProperties"], json!(false));
    assert_eq!(details["required"].as_array().unwrap().len(), 5);
}

#[test]
fn repeated_words_appear_once_among_the_allowed_hints() {
    let schema = phrase_schema("Could you send me the agenda before the call?").unwrap();
    assert_eq!(
        schema["properties"]["hint"]["enum"],
        json!([
            "Could", "you", "send", "me", "the", "agenda", "before", "call"
        ])
    );
    let mut inputs = Vec::new();
    for (source, text) in [
        ("one", "That's that, isn't it? That's all."),
        ("two", "reluctant"),
    ] {
        inputs.push(
            serde_json::from_value(json!({"source_id":source,"text":text,"reference_audio_id":null,"audio_sha256":null,"expected_revision":null}))
                .unwrap(),
        );
    }
    let batch = batch_schema(&inputs).unwrap();
    for item in batch["properties"]["phrases"]["items"]["anyOf"]
        .as_array()
        .unwrap()
    {
        let hints = item["properties"]["hint"]["enum"].as_array().unwrap();
        let distinct: std::collections::BTreeSet<_> =
            hints.iter().map(ToString::to_string).collect();
        assert_eq!(distinct.len(), hints.len(), "{hints:?}");
    }
}
