use phrase_gym_coach::transfer::{material::RawLesson, pending};

#[test]
fn raw_material_rejects_missing_meaning_urls_untrimmed_text_and_unsupported_locale() {
    let raw: RawLesson =
        serde_json::from_str(include_str!("../../../fixtures/coach/raw-english.json")).unwrap();
    raw.validate().unwrap();
    for invalid in [
        RawLesson {
            phrase: "https://example.com".into(),
            ..raw.clone()
        },
        RawLesson {
            situation: String::new(),
            ..raw.clone()
        },
        RawLesson {
            phrase: " Leading space".into(),
            ..raw.clone()
        },
        RawLesson {
            locale: "de-DE".into(),
            ..raw.clone()
        },
        RawLesson {
            phrase: "x".repeat(201),
            ..raw.clone()
        },
    ] {
        assert!(invalid.validate().is_err());
    }
}

#[cfg(unix)]
#[test]
fn deliberately_selected_symlinks_never_read_unrelated_bytes() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("private.wav");
    std::fs::write(&original, b"unrelated synthetic bytes").unwrap();
    let link = root.path().join("dropped.wav");
    std::os::unix::fs::symlink(&original, &link).unwrap();
    assert!(phrase_gym_coach::platform::files::read_selected(&link, pending::MAX_AUDIO).is_err());
    assert_eq!(
        phrase_gym_coach::platform::files::read_selected(&original, pending::MAX_AUDIO).unwrap(),
        b"unrelated synthetic bytes"
    );
}
