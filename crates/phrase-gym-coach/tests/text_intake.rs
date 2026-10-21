use phrase_gym_coach::transfer::text::TextDraft;
use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::media::{MediaOrigin, MediaRole};

#[test]
fn text_is_retained_before_speech_and_retries_keep_identity_without_accepting_changed_input() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("text.json");
    let first = TextDraft::retain(&path, "http://127.0.0.1:8000", "Let's get started.").unwrap();
    let again = TextDraft::retain(&path, &first.server, &first.phrase).unwrap();
    assert_eq!(first.submission_id, again.submission_id);
    assert_eq!(first.claim_id, again.claim_id);
    assert_eq!(
        TextDraft::retain(&path, "http://127.0.0.1:8000/", &first.phrase)
            .unwrap()
            .submission_id,
        first.submission_id
    );
    assert!(TextDraft::retain(&path, &first.server, "Another phrase.").is_err());
    assert!(TextDraft::retain(&path, "http://127.0.0.1:8001", &first.phrase).is_err());
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(saved["phrase"], first.phrase);
    assert!(!saved.to_string().contains("audio"));
}

#[test]
fn generated_media_keeps_exact_text_identity_and_never_claims_a_supplied_performer() {
    let root = tempfile::tempdir().unwrap();
    let draft = TextDraft::new("http://127.0.0.1:8000", "Let's get started.").unwrap();
    let voice = phrase_gym_coach::speech::PiperVoice {
        executable: root.path().join("synthetic python"),
        runtime_version: "1.8.0".into(),
        model: root.path().join("synthetic model"),
        config: root.path().join("synthetic config"),
        model_sha256: "a".repeat(64),
        config_sha256: "b".repeat(64),
        voice: "ljspeech-medium".into(),
        locale: "en-US".into(),
    };
    let metadata = phrase_gym_coach::speech::SpeechRequest {
        language: Language::En,
        text: draft.phrase.clone(),
        speech_text: draft.phrase.clone(),
        source: "synthetic".into(),
        role: MediaRole::Reference,
    }
    .metadata(&voice)
    .unwrap();
    let retained = draft.with_speech(b"fake bytes", metadata.clone()).unwrap();
    assert_eq!(retained.submission_id, draft.submission_id);
    assert_eq!(retained.upload.metadata.origin, MediaOrigin::Generated);
    assert_eq!(retained.upload.metadata.performer, None);
    retained
        .verify(&draft.server, uuid::Uuid::new_v4())
        .unwrap();
}

#[test]
fn unsupported_raw_inputs_fail_without_creating_a_draft() {
    for phrase in ["", "https://example.com", "two\twords"] {
        assert!(TextDraft::new("http://127.0.0.1:8000", phrase).is_err());
    }
}

#[test]
fn missing_speech_setup_preserves_retained_original_text_before_synthesis() {
    let root = tempfile::tempdir().unwrap();
    let paths = phrase_gym_coach::platform::paths::CoachPaths {
        config: root.path().join("config"),
        cache: root.path().join("cache"),
        logs: root.path().join("logs"),
        state: root.path().join("state"),
    };
    paths.ensure().unwrap();
    let draft = TextDraft::retain(
        &paths.state.join("pending-text.json"),
        "http://127.0.0.1:8000",
        "Let's get started.",
    )
    .unwrap();
    let problem = phrase_gym_coach::transfer::text::synthesize(
        &paths,
        &draft,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .err()
    .unwrap();
    assert!(problem.contains("piper-voice.json"));
    assert!(paths.state.join("pending-text.json").is_file());
    assert!(!paths.state.join("pending-send.json").exists());
    assert!(std::fs::read_dir(&paths.cache).unwrap().next().is_none());
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(paths.state.join("pending-text.json")).unwrap())
            .unwrap();
    assert_eq!(saved["phrase"], "Let's get started.");
}

#[test]
fn a_working_set_retains_frames_words_and_every_item_in_order() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/coach/working-set.json")).unwrap();
    let input = fixture["input"].as_str().unwrap();
    let draft = TextDraft::new("http://127.0.0.1:8000", input).unwrap();
    assert_eq!(draft.phrase, input);
    let entries = phrase_gym_coach::transfer::text::entries(input).unwrap();
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[3], "Could we move the meeting to Friday?");
    assert!(entries[4].starts_with("clarify"));
    assert!(phrase_gym_coach::transfer::text::entries(&format!("{input}\nA sixth item.")).is_err());
}
