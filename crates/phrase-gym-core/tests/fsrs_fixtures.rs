use std::path::PathBuf;

use chrono::{DateTime, Utc};
use phrase_gym_contracts::practice::RecallRating;
use phrase_gym_core::scheduler::{MemoryState, POLICY_VERSION, ReviewPolicy};
use phrase_gym_core::timestamps;
use serde_json::Value;

// Stability and difficulty may differ from the reference by floating-point noise only.
const RELATIVE_TOLERANCE: f64 = 1e-9;

fn fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/preservation/fsrs/review-policy.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn rating(name: &str) -> RecallRating {
    serde_json::from_value(Value::String(name.to_owned())).unwrap()
}

fn moment(value: &Value) -> DateTime<Utc> {
    timestamps::parse(value.as_str().unwrap()).unwrap()
}

// Reads the stored decimal text directly, so a lossy JSON float parser cannot hide a drift.
fn raw_number(text: &str, field: &str) -> Option<f64> {
    let start = text.find(&format!("\"{field}\": "))? + field.len() + 4;
    let end = start + text[start..].find([',', '}'])?;
    text[start..end].trim().parse().ok()
}

fn close(actual: Option<f64>, expected: Option<f64>, context: &str) {
    match (actual, expected) {
        (Some(actual), Some(expected)) => {
            let bound = RELATIVE_TOLERANCE * expected.abs().max(1.0);
            assert!(
                (actual - expected).abs() <= bound,
                "{context}: {actual} differs from {expected}"
            );
        }
        _ => assert_eq!(actual, expected, "{context}"),
    }
}

fn assert_equivalent(actual: &MemoryState, expected: &MemoryState, context: &str) {
    assert_eq!(actual.card_id, expected.card_id, "{context}");
    assert_eq!(actual.state, expected.state, "{context}");
    assert_eq!(actual.step, expected.step, "{context}");
    assert_eq!(actual.due, expected.due, "{context}: due");
    assert_eq!(actual.last_review, expected.last_review, "{context}");
    close(
        actual.stability,
        expected.stability,
        &format!("{context}: stability"),
    );
    close(
        actual.difficulty,
        expected.difficulty,
        &format!("{context}: difficulty"),
    );
}

#[test]
fn policy_identity_and_parameters_match_the_reference() {
    let document = fixture();
    assert_eq!(document["policy_version"], POLICY_VERSION);
    let policy = ReviewPolicy::default();
    let parameters = policy.scheduler().parameters();
    let scheduler = &document["scheduler"];
    let weights: Vec<f64> = serde_json::from_value(scheduler["parameters"].clone()).unwrap();
    assert_eq!(weights, parameters.weights.to_vec());
    assert_eq!(scheduler["desired_retention"], parameters.desired_retention);
    assert_eq!(scheduler["maximum_interval"], parameters.maximum_interval);
    assert_eq!(scheduler["learning_steps"], Value::Array(vec![]));
    assert_eq!(scheduler["relearning_steps"], Value::Array(vec![]));
    assert_eq!(scheduler["enable_fuzzing"], false);
    for (name, value) in document["ratings"].as_object().unwrap() {
        let expected = match rating(name) {
            RecallRating::NotYet => 1,
            RecallRating::WithEffort => 2,
            RecallRating::Recalled => 3,
        };
        assert_eq!(value, expected, "{name}");
    }
}

#[test]
fn each_review_from_a_stored_reference_state_matches() {
    let policy = ReviewPolicy::default();
    for case in fixture()["cases"].as_array().unwrap() {
        let mut stored: Option<String> = None;
        for (index, review) in case["reviews"].as_array().unwrap().iter().enumerate() {
            let context = format!("{} #{index}", case["name"]);
            let (state, due) = policy
                .review(
                    stored.as_deref(),
                    rating(review["rating"].as_str().unwrap()),
                    moment(&review["at"]),
                )
                .unwrap();
            let expected = MemoryState::from_json(review["state"].as_str().unwrap()).unwrap();
            assert_equivalent(
                &MemoryState::from_json(&state).unwrap(),
                &expected,
                &context,
            );
            assert_eq!(due, moment(&review["due"]), "{context}");
            stored = Some(review["state"].as_str().unwrap().to_owned());
        }
    }
}

#[test]
fn chained_sequences_never_drift_to_another_due_time() {
    let policy = ReviewPolicy::default();
    for case in fixture()["cases"].as_array().unwrap() {
        let mut state: Option<String> = None;
        for (index, review) in case["reviews"].as_array().unwrap().iter().enumerate() {
            let context = format!("{} chained #{index}", case["name"]);
            let (next, due) = policy
                .review(
                    state.as_deref(),
                    rating(review["rating"].as_str().unwrap()),
                    moment(&review["at"]),
                )
                .unwrap();
            let expected = MemoryState::from_json(review["state"].as_str().unwrap()).unwrap();
            assert_equivalent(&MemoryState::from_json(&next).unwrap(), &expected, &context);
            assert_eq!(due, moment(&review["due"]), "{context}");
            state = Some(next);
        }
    }
}

#[test]
fn stored_states_round_trip_without_changing_values() {
    for case in fixture()["cases"].as_array().unwrap() {
        for review in case["reviews"].as_array().unwrap() {
            let text = review["state"].as_str().unwrap();
            let parsed = MemoryState::from_json(text).unwrap();
            assert_eq!(MemoryState::from_json(&parsed.to_json()).unwrap(), parsed);
            let original: Value = serde_json::from_str(text).unwrap();
            let rewritten: Value = serde_json::from_str(&parsed.to_json()).unwrap();
            for field in ["card_id", "state", "step", "stability", "difficulty"] {
                assert_eq!(rewritten[field], original[field], "{field}");
            }
            assert_eq!(parsed.stability, raw_number(text, "stability"), "{text}");
            assert_eq!(parsed.difficulty, raw_number(text, "difficulty"), "{text}");
            for field in ["due", "last_review"] {
                assert_eq!(
                    moment(&rewritten[field]),
                    moment(&original[field]),
                    "{field}"
                );
            }
        }
    }
}
