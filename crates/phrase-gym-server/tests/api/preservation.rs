use serde_json::json;

use crate::support::scenario::{load, run, scenario_names};
use crate::support::setup::{DataFile, Setup, check_files, prepare};

#[tokio::test]
async fn rust_server_preserves_ported_scenarios() {
    let ported: Vec<String> = scenario_names()
        .into_iter()
        .filter(|name| load(name).implementations.iter().any(|item| item == "rust"))
        .collect();
    assert!(!ported.is_empty());
    for name in &ported {
        run(name).await;
    }
}

#[test]
fn every_scenario_is_readable_and_declares_its_implementations() {
    for name in scenario_names() {
        let scenario = load(&name);
        assert!(!scenario.description.is_empty(), "{name}");
        assert!(
            scenario
                .implementations
                .iter()
                .all(|item| item == "python" || item == "rust"),
            "{name}"
        );
    }
}

#[test]
fn scenario_setup_seeds_ages_removes_and_checks_data_files() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("unused.sqlite3");
    let seed: Setup = serde_json::from_value(json!({"files": [
        {"path": "staging/old.part", "bytes": {"text": "partial"}, "age_seconds": 7200},
        {"path": "media/original", "bytes": {"text": "original"}},
    ]}))
    .unwrap();
    prepare(&seed, directory.path(), &database);
    let age = std::fs::metadata(directory.path().join("staging/old.part"))
        .unwrap()
        .modified()
        .unwrap()
        .elapsed()
        .unwrap();
    assert!(age.as_secs() >= 7200, "{age:?}");
    let removal: Setup =
        serde_json::from_value(json!({"files": [{"path": "media/original", "absent": true}]}))
            .unwrap();
    prepare(&removal, directory.path(), &database);
    let expected: Vec<DataFile> = serde_json::from_value(json!([
        {"path": "staging/old.part", "bytes": {"text": "partial"}},
        {"path": "media/original", "absent": true},
    ]))
    .unwrap();
    check_files("setup", &expected, directory.path());
    assert!(!database.exists());
}
