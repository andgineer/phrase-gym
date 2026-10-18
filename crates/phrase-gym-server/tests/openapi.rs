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
    let mut checked = 0;
    for (path, operations) in document["paths"].as_object().unwrap() {
        for (method, operation) in operations.as_object().unwrap() {
            for (status, response) in operation["responses"].as_object().unwrap() {
                let failure = status.starts_with('4') || status.starts_with('5');
                if failure && status != "416" {
                    assert!(
                        response["content"]["application/json"].is_object(),
                        "{method} {path} {status}"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(checked >= 12, "{checked}");
    for schema in [
        "ExposureInput",
        "MediaUpload",
        "MediaMetadata",
        "TargetInput",
        "Cue",
    ] {
        assert_eq!(
            document["components"]["schemas"][schema]["additionalProperties"], false,
            "{schema}"
        );
    }
}

#[test]
fn test_only_seeding_routes_are_not_part_of_the_contract() {
    let document: serde_json::Value = serde_json::from_str(&openapi::document_json()).unwrap();
    assert!(
        document["paths"]
            .as_object()
            .unwrap()
            .keys()
            .all(|path| !path.starts_with("/api/v1/imports") && !path.starts_with("/_test/"))
    );
}
