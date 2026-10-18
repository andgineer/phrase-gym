use axum::http::StatusCode;
use chrono::DateTime;
use serde_json::{Value, json};

use crate::support::TestServer;
use crate::support::content::{
    WAV, get, package, package_text, phrases, post, post_text, publish, stage, upload,
};

fn server() -> TestServer {
    TestServer::start(
        DateTime::parse_from_rfc3339("2026-10-12T12:00:00Z")
            .unwrap()
            .to_utc(),
    )
}

fn count(server: &TestServer, table: &str) -> i64 {
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

fn identities(items: &[Value]) -> Vec<(String, String, i64)> {
    let mut found: Vec<_> = items
        .iter()
        .map(|item| {
            (
                item["content"]["source_id"].as_str().unwrap().to_owned(),
                item["id"].as_str().unwrap().to_owned(),
                item["revision"].as_i64().unwrap(),
            )
        })
        .collect();
    found.sort();
    found
}

#[tokio::test]
async fn reordering_targets_and_renaming_a_lesson_keep_every_identity() {
    let server = server();
    let mut content = package();
    publish(&server.router, &content).await;
    let original = identities(&phrases(&server.router, "en", "").await);
    let lessons = get(&server.router, "/api/v1/lessons?language=en")
        .await
        .json();
    content["targets"].as_array_mut().unwrap().reverse();
    content["lessons"][0]["title"] = json!("Rapport, renamed");
    content["lessons"][0]["target_source_ids"]
        .as_array_mut()
        .unwrap()
        .reverse();
    content["source_revision"] = json!("2");
    let report = publish(&server.router, &content).await;
    assert!(
        report["changes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|change| change["action"] == "unchanged")
    );
    assert_eq!(
        identities(&phrases(&server.router, "en", "").await),
        original
    );
    let renamed = get(&server.router, "/api/v1/lessons?language=en&q=renamed")
        .await
        .json();
    assert!(
        lessons["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|lesson| lesson["id"] == renamed["items"][0]["id"])
    );
    assert_eq!(count(&server, "target_revision"), 3);
}

#[tokio::test]
async fn object_key_order_never_creates_a_revision_but_array_order_does() {
    let server = server();
    let original_order = package_text();
    let staged = post_text(&server.router, "/api/v1/imports", original_order)
        .await
        .json();
    let path = format!("/api/v1/imports/{}/publish", staged["id"].as_str().unwrap());
    assert_eq!(
        post(&server.router, &path, &json!({})).await.status,
        StatusCode::OK
    );
    let content = package();
    let sorted_keys = stage(&server.router, &content).await.json();
    assert!(
        sorted_keys["changes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|change| change["action"] == "unchanged"),
        "{sorted_keys}"
    );
    let mut swapped = content;
    swapped["targets"][0]["concepts"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let report = publish(&server.router, &swapped).await;
    assert_eq!(report["changes"][0]["action"], "update");
    let day = &phrases(&server.router, "en", "greetings").await[0];
    assert_eq!(day["revision"], 2);
}

#[tokio::test]
async fn an_editorial_revision_keeps_the_old_revision_and_rebinds_its_dialogue() {
    let server = server();
    let mut content = package();
    publish(&server.router, &content).await;
    let first = get(&server.router, "/api/v1/dialogues?language=en")
        .await
        .json()["items"][0]
        .clone();
    let previous = phrases(&server.router, "en", "coffee").await[0].clone();
    content["targets"][1]["explanations"]["en"]["usage"] = json!("An editorial revision.");
    publish(&server.router, &content).await;
    let latest = phrases(&server.router, "en", "coffee").await[0].clone();
    assert_eq!(
        (latest["id"].clone(), latest["revision"].clone()),
        (previous["id"].clone(), json!(2))
    );
    let old = get(
        &server.router,
        &format!(
            "/api/v1/phrases/{}?language=en&revision=1",
            latest["id"].as_str().unwrap()
        ),
    )
    .await
    .json();
    assert_eq!(old["content"], previous["content"]);
    let dialogue = first["id"].as_str().unwrap();
    let current = get(
        &server.router,
        &format!("/api/v1/dialogues/{dialogue}?language=en"),
    )
    .await
    .json();
    assert_eq!(
        (
            current["revision"].clone(),
            current["target_revisions"]["catch-up"].clone()
        ),
        (json!(2), json!(2))
    );
    let pinned = get(
        &server.router,
        &format!("/api/v1/dialogues/{dialogue}?language=en&revision=1"),
    )
    .await
    .json();
    assert_eq!(pinned, first);
    assert_eq!(count(&server, "target"), 3);
}

#[tokio::test]
async fn a_stale_lesson_edit_conflicts_without_writing() {
    let server = server();
    let mut content = package();
    publish(&server.router, &content).await;
    let mut older = content.clone();
    older["lessons"][0]["title"] = json!("Old pending edit");
    let staged = stage(&server.router, &older).await.json();
    content["lessons"][0]["title"] = json!("New lesson edit");
    publish(&server.router, &content).await;
    let path = format!("/api/v1/imports/{}/publish", staged["id"].as_str().unwrap());
    assert_eq!(
        post(&server.router, &path, &json!({})).await.status,
        StatusCode::CONFLICT
    );
    let lessons = get(&server.router, "/api/v1/lessons?language=en&q=New")
        .await
        .json();
    assert_eq!(lessons["total"], 1);
}

#[tokio::test]
async fn a_stale_dialogue_edit_conflicts_without_writing() {
    let server = server();
    let mut content = package();
    publish(&server.router, &content).await;
    let mut older = content.clone();
    older["dialogues"][0]["turns"][0]["text"] = json!("Old pending line.");
    let staged = stage(&server.router, &older).await.json();
    content["dialogues"][0]["turns"][0]["text"] = json!("New dialogue line.");
    publish(&server.router, &content).await;
    let path = format!("/api/v1/imports/{}/publish", staged["id"].as_str().unwrap());
    assert_eq!(
        post(&server.router, &path, &json!({})).await.status,
        StatusCode::CONFLICT
    );
    let dialogue = get(&server.router, "/api/v1/dialogues?language=en")
        .await
        .json()["items"][0]
        .clone();
    assert_eq!(
        (
            dialogue["revision"].clone(),
            dialogue["content"]["turns"][0]["text"].clone()
        ),
        (json!(2), json!("New dialogue line."))
    );
    assert_eq!(count(&server, "dialogue_revision"), 2);
}

#[tokio::test]
async fn reference_audio_damaged_after_staging_blocks_publication() {
    let server = server();
    let reference = upload(&server.router, &json!({}), WAV, "audio/wav").await;
    let mut content = package();
    content["targets"][0]["reference_audio_id"] = reference["id"].clone();
    let staged = stage(&server.router, &content).await.json();
    assert_eq!(staged["status"], "staged", "{staged}");
    let original = server
        .data_dir
        .join("media")
        .join(reference["sha256"].as_str().unwrap());
    let mut damaged = std::fs::read(&original).unwrap();
    damaged[0] ^= 0xff;
    std::fs::write(&original, damaged).unwrap();
    let id = staged["id"].as_str().unwrap();
    let reply = post(
        &server.router,
        &format!("/api/v1/imports/{id}/publish"),
        &json!({}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CONFLICT, "{:?}", reply.body);
    assert_eq!(count(&server, "target"), 0);
    let after = get(&server.router, &format!("/api/v1/imports/{id}"))
        .await
        .json();
    assert_eq!(after["status"], "staged");
}

#[tokio::test]
async fn a_failure_late_in_publication_rolls_back_the_whole_package() {
    let server = server();
    let staged = stage(&server.router, &package()).await.json();
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .execute_batch(
            "CREATE TRIGGER fail_publication BEFORE INSERT ON dialogue_revision \
             BEGIN SELECT RAISE(ABORT, 'simulated publication crash'); END;",
        )
        .unwrap();
    let id = staged["id"].as_str().unwrap();
    let path = format!("/api/v1/imports/{id}/publish");
    let failed = post(&server.router, &path, &json!({})).await;
    assert_eq!(failed.status, StatusCode::INTERNAL_SERVER_ERROR);
    for table in [
        "target",
        "target_revision",
        "source_item",
        "lesson",
        "lesson_membership",
        "dialogue",
    ] {
        assert_eq!(count(&server, table), 0, "{table}");
    }
    let after = get(&server.router, &format!("/api/v1/imports/{id}"))
        .await
        .json();
    assert_eq!(after["status"], "staged");
    rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .execute_batch("DROP TRIGGER fail_publication;")
        .unwrap();
    assert_eq!(
        post(&server.router, &path, &json!({})).await.status,
        StatusCode::OK
    );
    assert_eq!(count(&server, "target"), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_publication_consumes_a_staged_package_once() {
    let server = server();
    let staged = stage(&server.router, &package()).await.json();
    let path = format!("/api/v1/imports/{}/publish", staged["id"].as_str().unwrap());
    let attempts: Vec<_> = (0..6)
        .map(|_| {
            let router = server.router.clone();
            let path = path.clone();
            tokio::spawn(async move { post(&router, &path, &json!({})).await })
        })
        .collect();
    let mut bodies = Vec::new();
    for attempt in attempts {
        let reply = attempt.await.unwrap();
        assert_eq!(reply.status, StatusCode::OK);
        bodies.push(reply.json());
    }
    assert!(bodies.iter().all(|body| *body == bodies[0]));
    assert_eq!(count(&server, "target"), 3);
    assert_eq!(count(&server, "target_revision"), 3);
}
