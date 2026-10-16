use std::path::PathBuf;

use phrase_gym_server::openapi;

#[test]
fn committed_schema_matches_the_server_routes() {
    let committed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../frontend/tests/rust-contract/openapi.json");
    let committed = std::fs::read_to_string(committed)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(
        committed,
        openapi::document_json(),
        "run `cargo xtask api-types`"
    );
}

#[test]
fn every_documented_operation_declares_its_error_bodies() {
    let document: serde_json::Value = serde_json::from_str(&openapi::document_json()).unwrap();
    let exposure = &document["paths"]["/api/v1/exposures"]["post"];
    for status in ["200", "409", "422"] {
        assert!(
            exposure["responses"][status]["content"]["application/json"].is_object(),
            "{status}"
        );
    }
    assert_eq!(
        document["components"]["schemas"]["ExposureInput"]["additionalProperties"],
        false
    );
}
