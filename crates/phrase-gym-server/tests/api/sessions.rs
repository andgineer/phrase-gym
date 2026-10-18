use std::collections::BTreeSet;

use axum::http::StatusCode;
use phrase_gym_core::clock::Clock;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::TestServer;
use crate::support::content::{get, package, post, publish};
use crate::support::practice::{
    count, current, execute, moment, patch, ready, session, start, started,
};

fn server() -> TestServer {
    TestServer::start(moment("2026-10-12T12:00:00+00:00"))
}

fn ids(values: &Value, field: &str) -> BTreeSet<String> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.pointer(field).unwrap().as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn start_retries_return_the_session_and_changed_parameters_conflict_after_restart() {
    let server = server();
    ready(&server.router, &mut package()).await;
    let id = Uuid::new_v4();
    let payload = json!({"id": id, "language": "en"});
    let first = post(&server.router, "/api/v1/sessions", &payload).await;
    assert_eq!(first.status, StatusCode::OK, "{:?}", first.body);
    let explicit = json!({"id": id, "language": "en", "lesson_id": null, "dialogue_id": null});
    let retried = post(&server.router, "/api/v1/sessions", &explicit).await;
    assert_eq!(retried.json(), first.json());
    let changed = post(
        &server.router,
        "/api/v1/sessions",
        &json!({"id": id, "language": "sr"}),
    )
    .await;
    assert_eq!(changed.status, StatusCode::CONFLICT);
    let restarted = server.restarted();
    let after_restart = post(&restarted, "/api/v1/sessions", &payload).await;
    assert_eq!(after_restart.json(), first.json());
    assert_eq!(current(&restarted).await, first.json());
}

#[tokio::test]
async fn pause_resume_retry_and_stale_transitions_keep_the_manifest() {
    let server = server();
    ready(&server.router, &mut package()).await;
    let first = started(&server.router, &json!({})).await;
    let pause = json!({"expected_version": 1, "status": "paused"});
    let paused = patch(&server.router, &first["id"], &pause).await.json();
    assert_eq!(
        (&paused["status"], &paused["version"]),
        (&json!("paused"), &json!(2))
    );
    assert_eq!(paused["manifest"], first["manifest"]);
    assert_eq!(
        patch(&server.router, &first["id"], &pause).await.json(),
        paused
    );
    assert_eq!(
        start(&server.router, &json!({})).await.status,
        StatusCode::CONFLICT,
        "a paused session keeps the open slot"
    );
    let stale = json!({"expected_version": 1, "status": "active"});
    assert_eq!(
        patch(&server.router, &first["id"], &stale).await.status,
        StatusCode::CONFLICT
    );
    let resume = json!({"expected_version": 2, "status": "active"});
    let resumed = patch(&server.router, &first["id"], &resume).await.json();
    assert_eq!(resumed["version"], 3);
    assert_eq!(resumed["manifest"], first["manifest"]);
    assert_eq!(
        patch(&server.router, &first["id"], &pause).await.status,
        StatusCode::CONFLICT
    );
    let unknown = json!(Uuid::new_v4());
    assert_eq!(
        get(
            &server.router,
            &format!("/api/v1/sessions/{}", unknown.as_str().unwrap())
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        patch(&server.router, &unknown, &pause).await.status,
        StatusCode::NOT_FOUND
    );
    let completed = json!({"expected_version": 3, "status": "completed"});
    assert_eq!(
        patch(&server.router, &first["id"], &completed).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn explicit_takeover_supersedes_with_fresh_review_groups() {
    let server = server();
    ready(&server.router, &mut package()).await;
    let first = started(&server.router, &json!({})).await;
    assert_eq!(
        start(&server.router, &json!({})).await.status,
        StatusCode::CONFLICT
    );
    let unknown = json!({"takeover_session_id": Uuid::new_v4()});
    assert_eq!(
        start(&server.router, &unknown).await.status,
        StatusCode::CONFLICT
    );
    let takeover = json!({"takeover_session_id": first["id"]});
    let second = started(&server.router, &takeover).await;
    let old = session(&server.router, &first["id"]).await;
    assert_eq!(
        (&old["status"], &old["version"]),
        (&json!("superseded"), &json!(2))
    );
    assert_eq!(old["manifest"], first["manifest"]);
    let resume = json!({"expected_version": 2, "status": "active"});
    assert_eq!(
        patch(&server.router, &first["id"], &resume).await.status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        start(&server.router, &takeover).await.status,
        StatusCode::CONFLICT
    );
    assert_eq!(current(&server.router).await, second);
    let groups = |state: &Value| ids(&state["manifest"]["targets"], "/review_group_id");
    assert!(groups(&first).is_disjoint(&groups(&second)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn simultaneous_starts_open_one_session_and_the_schema_enforces_one_slot() {
    let server = server();
    ready(&server.router, &mut package()).await;
    let attempts: Vec<_> = (0..4)
        .map(|_| {
            let router = server.router.clone();
            tokio::spawn(async move { start(&router, &json!({})).await.status })
        })
        .collect();
    let mut statuses = Vec::new();
    for attempt in attempts {
        statuses.push(attempt.await.unwrap());
    }
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::OK)
            .count(),
        1
    );
    assert!(
        statuses
            .iter()
            .all(|status| [StatusCode::OK, StatusCode::CONFLICT].contains(status))
    );
    let duplicate = rusqlite::Connection::open(&server.database_path)
        .unwrap()
        .execute_batch(
            "INSERT INTO practice_session SELECT 'second', start_payload, manifest, 'paused', 1, \
             version, created_at, updated_at, queue, repairs, cursor, last_sequence \
             FROM practice_session",
        );
    assert!(duplicate.is_err());
}

#[tokio::test]
async fn manifests_pin_unique_targets_cues_groups_and_media_without_scheduling() {
    let server = server();
    let content = ready(&server.router, &mut package()).await;
    let state = started(&server.router, &json!({})).await;
    assert_eq!(
        moment(state["created_at"].as_str().unwrap()),
        server.clock.now()
    );
    let planned = &state["manifest"]["targets"];
    assert_eq!(ids(planned, "/phrase/id"), ids(&json!(content), "/id"));
    assert_eq!(
        planned.as_array().unwrap().len(),
        3,
        "shared appearances are one target"
    );
    assert_eq!(ids(planned, "/review_group_id").len(), 3);
    for item in planned.as_array().unwrap() {
        assert_eq!(
            (&item["schedule_version"], &item["eligibility"]),
            (&json!(0), &json!("new"))
        );
        let cue = &item["phrase"]["content"]["cues"][0];
        assert_eq!(item["cue_id"], cue["id"]);
        let required = ids(&item["phrase"]["content"]["required_chunks"], "");
        let visible = ids(&cue["visible_chunks"], "");
        assert_eq!(ids(&item["hidden_chunks"], ""), &required - &visible);
    }
    assert_eq!(
        ids(&state["manifest"]["required_media_ids"], ""),
        ids(&json!(content), "/content/reference_audio_id")
    );
    assert_eq!(state["manifest"]["planner_version"], "due-new-v3-online");
    assert_eq!(state["manifest"]["schema_version"], 1);
    assert_eq!(count(&server, "target_schedule"), 0);
}

#[tokio::test]
async fn due_targets_lead_future_ones_wait_and_a_lesson_scopes_the_plan() {
    let server = server();
    let content = ready(&server.router, &mut package()).await;
    let by_source = |source: &str| {
        content
            .iter()
            .find(|phrase| phrase["content"]["source_id"] == source)
            .unwrap()
            .clone()
    };
    let (day, rather) = (by_source("day"), by_source("rather"));
    execute(
        &server,
        &format!(
            "INSERT INTO target_schedule (target_id, version, due_at) VALUES \
             ('{}', 4, '2026-10-12T12:00:00+00:00'), ('{}', 2, '2026-10-13T12:00:00+00:00');",
            day["id"].as_str().unwrap(),
            rather["id"].as_str().unwrap()
        ),
    );
    let lesson = &day["lessons"][0]["id"];
    let scoped = started(&server.router, &json!({"lesson_id": lesson})).await;
    let manifest = &scoped["manifest"];
    assert_eq!(&manifest["lesson_id"], lesson);
    let eligibility: Vec<_> = manifest["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| &t["eligibility"])
        .collect();
    assert_eq!(eligibility, [&json!("due"), &json!("new")]);
    assert_eq!(manifest["targets"][0]["schedule_version"], 4);
    let everything = started(
        &server.router,
        &json!({"takeover_session_id": scoped["id"]}),
    )
    .await;
    let targets = everything["manifest"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 2, "the future review waits");
    assert_eq!(targets[0]["phrase"]["id"], day["id"]);
    let listed = get(
        &server.router,
        &format!(
            "/api/v1/lessons/{}/phrases?language=en",
            lesson.as_str().unwrap()
        ),
    )
    .await;
    assert_eq!(listed.json()["items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn unready_content_and_invalid_scopes_create_no_session() {
    let server = server();
    publish(&server.router, &package()).await;
    assert_eq!(
        start(&server.router, &json!({})).await.status,
        StatusCode::CONFLICT
    );
    let lesson = json!({"lesson_id": Uuid::new_v4()});
    assert_eq!(
        start(&server.router, &lesson).await.status,
        StatusCode::NOT_FOUND
    );
    let dialogue = json!({"dialogue_id": Uuid::new_v4()});
    assert_eq!(
        start(&server.router, &dialogue).await.status,
        StatusCode::NOT_FOUND
    );
    let both = json!({"lesson_id": Uuid::new_v4(), "dialogue_id": Uuid::new_v4()});
    assert_eq!(
        start(&server.router, &both).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let unknown_language = json!({"language": "fr"});
    assert_eq!(
        start(&server.router, &unknown_language).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(current(&server.router).await, Value::Null);
    let content = ready(&server.router, &mut package()).await;
    let other_language = json!({"language": "sr", "lesson_id": content[0]["lessons"][0]["id"]});
    assert_eq!(
        start(&server.router, &other_language).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(count(&server, "practice_session"), 0);
}

#[tokio::test]
async fn missing_reference_files_exclude_targets_and_failed_takeover_keeps_the_session() {
    let server = server();
    ready(&server.router, &mut package()).await;
    let first = started(&server.router, &json!({})).await;
    let wrong_language = json!({"language": "sr", "takeover_session_id": first["id"]});
    assert_eq!(
        start(&server.router, &wrong_language).await.status,
        StatusCode::CONFLICT
    );
    assert_eq!(current(&server.router).await, first);
    for entry in std::fs::read_dir(server.data_dir.join("media")).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    let takeover = json!({"takeover_session_id": first["id"]});
    assert_eq!(
        start(&server.router, &takeover).await.status,
        StatusCode::CONFLICT
    );
    assert_eq!(current(&server.router).await, first);
}

#[tokio::test]
async fn manifests_stay_immutable_after_publication_and_replacements_pin_new_revisions() {
    let server = server();
    let mut content = package();
    ready(&server.router, &mut content).await;
    let first = started(&server.router, &json!({})).await;
    content["source_revision"] = json!("2");
    content["targets"][0]["purpose"] = json!("A revised editorial purpose.");
    publish(&server.router, &content).await;
    get(&server.router, "/api/v1/phrases/search?language=sr").await;
    assert_eq!(session(&server.router, &first["id"]).await, first);
    assert_eq!(current(&server.router).await, first);
    let replacement = started(&server.router, &json!({"takeover_session_id": first["id"]})).await;
    let day = replacement["manifest"]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|target| target["phrase"]["content"]["source_id"] == "day")
        .unwrap();
    assert_eq!(day["phrase"]["revision"], 2);
}

fn numbered_package(count: usize) -> Value {
    let targets: Vec<Value> = (0..count)
        .map(|index| {
            json!({
                "source_id": format!("t{index:02}"), "kind": "fixed_phrase",
                "meaning_key": format!("k{index:02}"),
                "recall_expression": format!("Phrase {index} is ready."),
                "text": format!("Phrase {index} is ready."),
                "purpose": "A numbered synthetic phrase.",
                "chunks": [{"id": "opening", "text": format!("Phrase {index} ")},
                           {"id": "ending", "text": "is ready."}],
                "required_chunks": ["opening"],
                "cues": [{"id": "cue", "prompt": "Say the numbered phrase.",
                          "visible_chunks": ["ending"], "reviewed": true}],
            })
        })
        .collect();
    json!({"schema_version": 1, "namespace": "capacity", "language": "en",
           "source_revision": "1", "targets": targets})
}

#[tokio::test]
async fn plans_take_the_oldest_twenty_due_targets_and_at_most_five_new_ones() {
    let server = server();
    let phrases = ready(&server.router, &mut numbered_package(30)).await;
    let mut ordered: Vec<&Value> = phrases.iter().collect();
    ordered.sort_by_key(|phrase| phrase["content"]["source_id"].as_str().unwrap().to_owned());
    // Target tNN falls due at minute NN, so lower numbers are older reviews.
    let schedule = |targets: &[&Value]| {
        targets
            .iter()
            .map(|phrase| {
                let source = phrase["content"]["source_id"].as_str().unwrap();
                format!(
                    "('{}', 1, '2026-10-11T08:{}:00+00:00')",
                    phrase["id"].as_str().unwrap(),
                    &source[1..],
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let fresh = started(&server.router, &json!({})).await;
    assert_eq!(fresh["manifest"]["targets"].as_array().unwrap().len(), 5);
    execute(
        &server,
        &format!(
            "INSERT INTO target_schedule (target_id, version, due_at) VALUES {};",
            schedule(&ordered[..17])
        ),
    );
    let mixed = started(&server.router, &json!({"takeover_session_id": fresh["id"]})).await;
    let eligibility: Vec<String> = mixed["manifest"]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|target| target["eligibility"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(eligibility.len(), 20);
    assert!(eligibility[..17].iter().all(|kind| kind == "due"));
    assert!(eligibility[17..].iter().all(|kind| kind == "new"));
    execute(
        &server,
        &format!(
            "INSERT INTO target_schedule (target_id, version, due_at) VALUES {};",
            schedule(&ordered[17..25])
        ),
    );
    let backlog = started(&server.router, &json!({"takeover_session_id": mixed["id"]})).await;
    let targets = backlog["manifest"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 20);
    assert!(targets.iter().all(|target| target["eligibility"] == "due"));
    let due: Vec<_> = targets
        .iter()
        .map(|target| target["phrase"]["content"]["source_id"].as_str().unwrap())
        .collect();
    let oldest: Vec<String> = (0..20).map(|index| format!("t{index:02}")).collect();
    assert_eq!(due, oldest);
}
