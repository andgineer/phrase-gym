use axum::http::StatusCode;
use chrono::DateTime;
use serde_json::{Value, json};

use crate::support::TestServer;
use crate::support::content::{WAV, package, phrases, post, publish, stage, upload};

fn server() -> TestServer {
    TestServer::start(
        DateTime::parse_from_rfc3339("2026-10-12T12:00:00Z")
            .unwrap()
            .to_utc(),
    )
}

fn broken(mutation: &str) -> Value {
    let mut content = package();
    let target = &mut content["targets"][0];
    match mutation {
        "unknown_chunk" => target["cues"][0]["visible_chunks"]
            .as_array_mut()
            .unwrap()
            .push(json!("absent")),
        "joined_text" => target["chunks"][0]["text"] = json!("Wrong "),
        "unreviewed" => target["cues"][0]["reviewed"] = json!(false),
        "unhidden" => target["cues"][0]["visible_chunks"] = json!(["opening", "subject", "ending"]),
        "slot" => target["slots"] = json!({"absent": ["replacement"]}),
        "required" => target["required_chunks"]
            .as_array_mut()
            .unwrap()
            .push(json!("absent")),
        "duplicate_required" => target["required_chunks"]
            .as_array_mut()
            .unwrap()
            .push(json!("opening")),
        "duplicate_cue" => {
            let cue = target["cues"][0].clone();
            target["cues"].as_array_mut().unwrap().push(cue);
        }
        "bad_identifier" => target["source_id"] = json!("has space"),
        "empty_text" => target["purpose"] = json!(""),
        "lesson_member" => content["lessons"][0]["target_source_ids"]
            .as_array_mut()
            .unwrap()
            .push(json!("absent")),
        "dialogue_text" => content["dialogues"][0]["turns"][1]["text"] = json!("Different phrase"),
        "dialogue_target" => {
            content["dialogues"][0]["turns"][1]["target_source_id"] = json!("absent");
        }
        "unknown_field" => content["reviewer"] = json!("someone"),
        "schema_version" => content["schema_version"] = json!(2),
        duplicate => {
            let collection = duplicate.trim_start_matches("duplicate_");
            let first = content[collection][0].clone();
            content[collection].as_array_mut().unwrap().push(first);
        }
    }
    content
}

#[tokio::test]
async fn malformed_packages_are_invalid_requests_and_publish_nothing() {
    let server = server();
    publish(&server.router, &package()).await;
    let previous = phrases(&server.router, "en", "").await;
    for (mutation, message) in [
        (
            "unknown_chunk",
            "Visible chunks must be distinct existing chunk IDs",
        ),
        ("joined_text", "chunks must concatenate to the phrase"),
        ("unreviewed", "reviewed cues only"),
        ("unhidden", "must hide required material"),
        ("slot", "Frame slots"),
        ("required", "Required chunks"),
        ("duplicate_required", "Required chunks"),
        ("duplicate_cue", "Cue IDs"),
        ("bad_identifier", "source_id must be"),
        ("empty_text", "purpose must be"),
        ("lesson_member", "Lesson members"),
        ("dialogue_text", "Dialogue target text must match"),
        ("dialogue_target", "Dialogue target must exist"),
        ("duplicate_targets", "Target source IDs"),
        ("duplicate_lessons", "Lesson/dialogue source IDs"),
        ("duplicate_dialogues", "Lesson/dialogue source IDs"),
        ("unknown_field", "reviewer"),
        ("schema_version", "schema_version"),
    ] {
        let reply = stage(&server.router, &broken(mutation)).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{mutation}");
        let detail = reply.json()["detail"].as_str().unwrap().to_owned();
        assert!(detail.contains(message), "{mutation}: {detail}");
    }
    assert_eq!(phrases(&server.router, "en", "").await, previous);
}

#[tokio::test]
async fn changed_objectives_and_group_languages_need_new_source_identities() {
    let server = server();
    publish(&server.router, &package()).await;
    let previous = phrases(&server.router, "en", "").await;
    for (field, value) in [
        ("meaning_key", "different-sense"),
        ("kind", "word_sense"),
        ("recall_expression", "another expression"),
    ] {
        let mut content = package();
        content["targets"][0][field] = json!(value);
        let report = stage(&server.router, &content).await.json();
        assert_eq!(report["status"], "rejected", "{field}");
        assert_eq!(report["id"], Value::Null);
        assert!(
            report["errors"][0]
                .as_str()
                .unwrap()
                .contains("new source ID")
        );
    }
    for kind in ["lesson", "dialogue"] {
        let mut content = package();
        content["language"] = json!("sr");
        content["namespace"] = json!("manual.examples.en");
        for target in content["targets"].as_array_mut().unwrap() {
            target["source_id"] = json!(format!("new-{}", target["source_id"].as_str().unwrap()));
        }
        for lesson in content["lessons"].as_array_mut().unwrap() {
            let members: Vec<String> = lesson["target_source_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|source| format!("new-{}", source.as_str().unwrap()))
                .collect();
            lesson["target_source_ids"] = json!(members);
            if kind != "lesson" {
                lesson["source_id"] =
                    json!(format!("new-{}", lesson["source_id"].as_str().unwrap()));
            }
        }
        let dialogue = &mut content["dialogues"][0];
        if kind != "dialogue" {
            dialogue["source_id"] = json!("new-coffee");
        }
        dialogue["turns"][1]["target_source_id"] = json!("new-catch-up");
        let report = stage(&server.router, &content).await.json();
        assert_eq!(report["status"], "rejected", "{kind}");
        let expected = format!("{}{} ", kind[..1].to_uppercase(), &kind[1..]);
        assert!(
            report["errors"].as_array().unwrap().iter().any(|error| {
                let error = error.as_str().unwrap();
                error.starts_with(&expected) && error.contains("language changes")
            }),
            "{report}"
        );
    }
    assert_eq!(phrases(&server.router, "en", "").await, previous);
    assert!(phrases(&server.router, "sr", "").await.is_empty());
}

#[tokio::test]
async fn bound_audio_must_exist_and_match_the_exact_language_text_and_role() {
    let server = server();
    publish(&server.router, &package()).await;
    let unknown = "0000dead-0000-4000-8000-000000000009";
    let lost = upload(&server.router, &json!({}), WAV, "audio/wav").await;
    std::fs::remove_file(
        server
            .data_dir
            .join("media")
            .join(lost["sha256"].as_str().unwrap()),
    )
    .unwrap();
    for media in [json!(unknown), lost["id"].clone()] {
        let mut content = package();
        content["targets"][0]["reference_audio_id"] = media;
        let report = stage(&server.router, &content).await.json();
        assert_eq!(report["status"], "rejected");
        assert!(
            report["errors"][0]
                .as_str()
                .unwrap()
                .contains("missing reference media")
        );
    }
    for overrides in [
        json!({"text": "day"}),
        json!({"language": "sr", "locale": "sr-RS"}),
        json!({"role": "cue"}),
    ] {
        let asset = upload(&server.router, &overrides, WAV, "audio/wav").await;
        let mut content = package();
        content["targets"][0]["reference_audio_id"] = asset["id"].clone();
        let report = stage(&server.router, &content).await.json();
        assert_eq!(report["status"], "rejected", "{overrides}");
        assert!(
            report["errors"][0]
                .as_str()
                .unwrap()
                .contains("exact language, text and role")
        );
    }
    let whole_phrase_cue = upload(&server.router, &json!({"role": "cue"}), WAV, "audio/wav").await;
    let mut content = package();
    content["targets"][0]["cues"][0]["audio_id"] = whole_phrase_cue["id"].clone();
    assert_eq!(
        stage(&server.router, &content).await.json()["status"],
        "rejected"
    );
    let hint = upload(
        &server.router,
        &json!({"role": "cue", "text": "your day been so far?"}),
        WAV,
        "audio/wav",
    )
    .await;
    content["targets"][0]["cues"][0]["audio_id"] = hint["id"].clone();
    assert_eq!(
        stage(&server.router, &content).await.json()["status"],
        "staged"
    );
}

#[tokio::test]
async fn cues_without_phrase_specific_visible_text_are_rejected() {
    let server = server();
    for visible in [json!([]), json!(["punctuation"]), json!(["whitespace"])] {
        let mut content = package();
        let target = &mut content["targets"][0];
        target["chunks"] = json!([
            {"id": "opening", "text": "How has"},
            {"id": "whitespace", "text": " "},
            {"id": "subject", "text": "your day "},
            {"id": "ending", "text": "been so far"},
            {"id": "punctuation", "text": "?"},
        ]);
        target["cues"][0]["visible_chunks"] = visible.clone();
        let report = stage(&server.router, &content).await.json();
        assert_eq!(report["status"], "rejected", "{visible}");
        assert!(
            report["errors"][0]
                .as_str()
                .unwrap()
                .contains("visible phrase-specific text")
        );
    }
    let validated = post(&server.router, "/api/v1/imports/validate", &package())
        .await
        .json();
    assert_eq!(
        (validated["status"].clone(), validated["id"].clone()),
        (json!("validated"), Value::Null)
    );
    assert!(phrases(&server.router, "en", "").await.is_empty());
}
