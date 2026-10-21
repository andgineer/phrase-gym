use phrase_gym_contracts::english::{EnglishTask, PreparedEnglishLesson};
use serde_json::json;

use super::support::Fixture;

fn word(fixture: &Fixture) -> PreparedEnglishLesson {
    let mut package = fixture.package();
    let phrase = &mut package.phrases[0];
    let mut contexts = Vec::new();
    let prompts = ["Please _____ the deadline.", "Can you _____ this point?"];
    phrase.situations = prompts.iter().enumerate().map(|(index, prompt)| {
        let answer = prompt.replace("_____", "clarify");
        let reference = fixture.upload(&json!({"text": answer}));
        contexts.push(json!({"situation_id":format!("context-{index}"), "answer":answer, "reference_audio_id":reference.id}));
        serde_json::from_value(json!({"id":format!("context-{index}"), "text":prompt, "language":"en", "audio_id":null, "partner":null, "exposed_target_source_ids":[]})).unwrap()
    }).collect();
    phrase.source_text = Some("clarify".into());
    phrase.text = "Please clarify the deadline.".into();
    phrase.reference_audio_id = fixture.upload(&json!({"text": phrase.text})).id;
    phrase.hint = "cl".into();
    phrase.accepted_variants.clear();
    phrase.task = Some(serde_json::from_value(json!({"kind":"word_sense", "word":"clarify", "sense":"Make clearer", "contexts": contexts})).unwrap());
    package
}

#[test]
fn word_contexts_bind_distinct_gap_answers_and_exact_references_without_answer_leaks() {
    let fixture = Fixture::new();
    let package = word(&fixture);
    let mut wrong_task = package.clone();
    wrong_task.phrases[0].task = None;
    assert!(fixture.validator.validate(&wrong_task).is_err());
    fixture.validator.validate(&package).unwrap();
    for change in [
        "leak",
        "wrong-answer",
        "truncated-answer",
        "truncated-inside-word",
        "wrong-audio",
        "missing-audio",
        "missing-context",
        "full-hint",
    ] {
        let mut invalid = package.clone();
        let phrase = &mut invalid.phrases[0];
        let Some(EnglishTask::WordSense { contexts, .. }) = &mut phrase.task else {
            unreachable!()
        };
        match change {
            "leak" => phrase.situations[0].text.push_str(" Use clarify."),
            "wrong-answer" => contexts[1].answer = "A different sentence with clarify.".into(),
            "truncated-answer" | "truncated-inside-word" => {
                contexts[1].answer = if change == "truncated-answer" {
                    "Can you clarify"
                } else {
                    "Can you clar"
                }
                .into();
                contexts[1].reference_audio_id =
                    fixture.upload(&json!({"text":contexts[1].answer})).id;
            }
            "wrong-audio" => contexts[1].reference_audio_id = contexts[0].reference_audio_id,
            "missing-audio" => contexts[1].reference_audio_id = uuid::Uuid::new_v4(),
            "missing-context" => {
                contexts.pop();
            }
            "full-hint" => phrase.hint = "clarify".into(),
            _ => unreachable!(),
        }
        assert!(fixture.validator.validate(&invalid).is_err(), "{change}");
    }
}

#[test]
fn an_explicit_trailing_sense_cue_is_outside_the_complete_spoken_answer() {
    let fixture = Fixture::new();
    let mut package = word(&fixture);
    package.phrases[0].situations[0]
        .text
        .push_str(" | Make it clear.");
    fixture.validator.validate(&package).unwrap();
    package.phrases[0].situations[0].text = "Please _____ the deadline. Make it clear.".into();
    assert!(fixture.validator.validate(&package).is_err());
}

#[test]
fn a_frame_is_a_declared_variable_construction_with_a_complete_verified_example() {
    let fixture = Fixture::new();
    let mut package = fixture.package();
    let phrase = &mut package.phrases[0];
    phrase.text = "Could we move the call to Friday?".into();
    phrase.reference_audio_id = fixture.upload(&json!({"text": phrase.text})).id;
    phrase.hint = "move".into();
    phrase.task = Some(serde_json::from_value(json!({"kind":"frame", "pattern":"Could we move {item} to {day}?", "slots":[{"name":"item","values":["the call","the meeting"]},{"name":"day","values":["Friday","Monday"]}]})).unwrap());
    fixture.validator.validate(&package).unwrap();
    for change in [
        "unlisted-example",
        "undeclared-slot",
        "one-value",
        "duplicate-name",
    ] {
        let mut invalid = package.clone();
        let phrase = &mut invalid.phrases[0];
        let Some(EnglishTask::Frame { pattern, slots }) = &mut phrase.task else {
            unreachable!()
        };
        match change {
            "unlisted-example" => phrase.text = "Could we cancel it?".into(),
            "undeclared-slot" => pattern.push_str(" {unknown}"),
            "one-value" => slots[0].values.truncate(1),
            "duplicate-name" => slots[1].name = slots[0].name.clone(),
            _ => unreachable!(),
        }
        assert!(fixture.validator.validate(&invalid).is_err(), "{change}");
    }
}
