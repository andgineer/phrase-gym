use crate::support::scenario::{load, run, scenario_names};

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
