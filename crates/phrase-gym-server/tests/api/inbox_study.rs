use axum::http::StatusCode;
use chrono::DateTime;
use serde_json::{Value, json};
use uuid::Uuid;

use super::inbox::{lesson, output, post};
use crate::support::TestServer;
use crate::support::content::get;

fn event(sequence: i64, kind: &str) -> Value {
    json!({"events": [{"id": Uuid::new_v4(), "sequence": sequence, "step": 0,
        "kind": kind, "occurred_at": "2026-10-19T10:00:00Z"}]})
}

#[tokio::test]
async fn normal_start_studies_english_without_inventing_memory_or_speech() {
    let server = TestServer::start(
        DateTime::parse_from_rfc3339("2026-10-19T10:00:00Z")
            .unwrap()
            .to_utc(),
    );
    let (input, _) = lesson(&server).await;
    let receipt = post(&server.router, "/api/v1/inbox/lessons", &input)
        .await
        .json();
    let receipt_path = format!(
        "/api/v1/inbox/submissions/{}",
        input["submission_id"].as_str().unwrap()
    );
    assert_eq!(
        get(&server.restarted(), &receipt_path).await.json(),
        receipt
    );
    let claim = post(
        &server.router,
        "/api/v1/preparation/claims",
        &json!({"claim_id": Uuid::new_v4()}),
    )
    .await
    .json();
    assert_eq!(
        post(&server.router, "/api/v1/preparation/stage", &output(&claim))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        post(
            &server.router,
            "/api/v1/preparation/publish",
            &claim["fence"]
        )
        .await
        .status,
        StatusCode::OK
    );
    let id = Uuid::new_v4();
    let session = post(
        &server.router,
        "/api/v1/sessions",
        &json!({"id": id, "language": "en"}),
    )
    .await;
    assert_eq!(session.status, StatusCode::OK, "{}", session.json());
    assert_eq!(
        session.json()["manifest"]["targets"][0]["eligibility"],
        "study"
    );
    assert_eq!(
        session.json()["manifest"]["targets"][0]["hidden_chunks"],
        json!([])
    );
    let path = format!("/api/v1/sessions/{id}/events");
    let presented = post(&server.router, &path, &event(1, "present"))
        .await
        .json();
    assert_eq!(presented["session"]["attempts"][0]["revealed"], true);
    assert_eq!(presented["session"]["attempts"][0]["supported"], true);
    let mut rate = event(2, "rate");
    rate["events"][0]["rating"] = "recalled".into();
    assert_eq!(
        post(&server.router, &path, &rate).await.status,
        StatusCode::CONFLICT
    );
    let next = event(2, "next");
    let completed = post(&server.router, &path, &next).await.json();
    assert_eq!(completed["session"]["status"], "completed");
    let attempt = &completed["session"]["attempts"][0];
    assert!(attempt["original"].is_null());
    assert!(attempt["rating"].is_null());
    assert_eq!(attempt["eligibility_reason"], "ungraded_study");
    assert_eq!(completed["receipts"][0]["schedule_changed"], false);
    let restarted = server.restarted();
    assert_eq!(post(&restarted, &path, &next).await.json(), completed);
    assert_eq!(
        get(&restarted, &format!("/api/v1/sessions/{id}"))
            .await
            .json(),
        completed["session"]
    );
    let again = post(
        &restarted,
        "/api/v1/sessions",
        &json!({"id": Uuid::new_v4(), "language": "en"}),
    )
    .await;
    assert_eq!(
        again.json()["manifest"]["targets"][0]["schedule_version"],
        0
    );
    provisional_schedule_does_not_block_first_independent_recall(&server, &again.json()).await;
}

async fn provisional_schedule_does_not_block_first_independent_recall(
    server: &TestServer,
    current: &Value,
) {
    let connection = rusqlite::Connection::open(&server.database_path).unwrap();
    let target = current["manifest"]["targets"][0]["phrase"]["id"]
        .as_str()
        .unwrap();
    let due = "2026-10-21T10:00:00+00:00";
    connection.execute(
        "INSERT INTO target_schedule (target_id, version, due_at, memory_state, policy_version) VALUES (?1, 7, ?2, NULL, ?3)",
        rusqlite::params![target, due, phrase_gym_core::scheduler::POLICY_VERSION],
    ).unwrap();
    let id = Uuid::new_v4();
    let restarted = server.restarted();
    let study = post(
        &restarted,
        "/api/v1/sessions",
        &json!({"id": id, "language": "en", "takeover_session_id": current["id"]}),
    )
    .await;
    assert_eq!(study.status, StatusCode::OK, "{}", study.json());
    assert_eq!(study.json()["manifest"]["targets"][0]["eligibility"], "new");
    let before: (i64, String, Option<String>) = connection
        .query_row(
            "SELECT version, due_at, memory_state FROM target_schedule WHERE target_id = ?1",
            [target],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(before, (7, due.into(), None));
    let path = format!("/api/v1/sessions/{id}/events");
    assert_eq!(
        post(&restarted, &path, &event(1, "present")).await.status,
        StatusCode::OK
    );
    assert_eq!(
        post(&restarted, &path, &event(2, "answer")).await.status,
        StatusCode::OK
    );
    let mut rating = event(3, "rate");
    rating["events"][0]["rating"] = "recalled".into();
    let finished = post(&restarted, &path, &rating).await.json();
    assert_eq!(finished["session"]["status"], "completed");
    assert_eq!(
        finished["session"]["attempts"][0]["original"]["supported"],
        false
    );
    assert_eq!(finished["receipts"][0]["schedule_changed"], true);
    assert_eq!(finished["receipts"][0]["eligibility_reason"], "eligible");
    let retained: (i64, String, Option<String>) = connection
        .query_row(
            "SELECT version, due_at, memory_state FROM target_schedule WHERE target_id = ?1",
            [target],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(retained.0, 8);
    assert!(retained.2.is_some());
}
