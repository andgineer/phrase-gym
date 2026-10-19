use phrase_gym_contracts::english::PartnerPrompt;
use serde_json::json;

use super::support::{ANSWER, Fixture};

#[test]
fn supplied_audio_requires_verified_bytes_and_exact_text_language_and_role() {
    let fixture = Fixture::new();
    let package = fixture.package();
    for overrides in [
        json!({"text": "A different answer"}),
        json!({"language": "de", "locale": "de-DE"}),
        json!({"role": "cue"}),
        json!({"role": "partner"}),
        json!({"origin": "generated", "engine": "fake", "voice": "fake", "speech_text": ANSWER}),
    ] {
        let mut changed = package.clone();
        changed.phrases[0].reference_audio_id = fixture.upload(&overrides).id;
        assert!(fixture.validator.validate(&changed).is_err());
    }
    let mut changed = package.clone();
    changed.phrases[0].reference_audio_id = uuid::Uuid::new_v4();
    assert!(fixture.validator.validate(&changed).is_err());
    let asset = fixture.upload(&json!({}));
    let path = fixture.directory.path().join("media").join(&asset.sha256);
    let mut bytes = std::fs::read(&path).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    std::fs::write(&path, bytes).unwrap();
    assert!(fixture.validator.validate(&package).is_err());
}

#[test]
fn situation_and_partner_audio_have_separate_exact_bindings() {
    let fixture = Fixture::new();
    let mut package = fixture.package();
    let situation = &mut package.phrases[0].situations[0];
    situation.audio_id = Some(
        fixture
            .upload(&json!({"text": situation.text, "role": "cue"}))
            .id,
    );
    let partner = "Is everyone here?";
    situation.partner = Some(PartnerPrompt {
        text: partner.into(),
        audio_id: Some(
            fixture
                .upload(&json!({"text": partner, "role": "partner"}))
                .id,
        ),
    });
    assert!(fixture.validator.validate(&package).is_ok());
    let mut wrong = package.clone();
    wrong.phrases[0].situations[0]
        .partner
        .as_mut()
        .unwrap()
        .audio_id = wrong.phrases[0].situations[0].audio_id;
    assert!(fixture.validator.validate(&wrong).is_err());
    let mut wrong = package.clone();
    wrong.phrases[0].situations[0].audio_id = Some(package.phrases[0].reference_audio_id);
    assert!(fixture.validator.validate(&wrong).is_err());
}
