use std::collections::BTreeMap;

use axum::body::Body;
use axum::http::Request;
use chrono::DateTime;
use http_body_util::BodyExt;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use tower::ServiceExt;

use super::documents::{expand, scenario_bytes, substitute};
use super::{TestServer, repository_root};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub description: String,
    pub implementations: Vec<String>,
    clock: String,
    #[serde(default)]
    seed_sql: Vec<String>,
    steps: Vec<Step>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    clock: Option<String>,
    request: Value,
    expect: Value,
    #[serde(default)]
    capture: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioRequest {
    method: String,
    path: String,
    json: Option<Value>,
    raw: Option<String>,
    #[serde(default)]
    headers: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expectation {
    status: u16,
    #[serde(default)]
    json: Vec<Check>,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    bytes: Option<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    at: String,
    #[serde(default, deserialize_with = "present")]
    equals: Option<Value>,
    same_as: Option<String>,
    instant: Option<String>,
    length: Option<usize>,
    #[serde(default)]
    absent: bool,
}

fn present<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Value>, D::Error> {
    Value::deserialize(deserializer).map(Some)
}

pub fn scenario_names() -> Vec<String> {
    let directory = repository_root().join("fixtures/preservation/scenarios");
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

pub fn load(name: &str) -> Scenario {
    let path = repository_root().join(format!("fixtures/preservation/scenarios/{name}.json"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn moment(text: &str) -> DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339(text).unwrap().to_utc()
}

fn check(step: &str, body: &Value, check: &Check, responses: &BTreeMap<String, Value>) {
    let place = format!("{step} {}", check.at);
    if check.absent {
        let (parent, key) = check.at.rsplit_once('/').unwrap();
        assert!(body.pointer(parent).unwrap().get(key).is_none(), "{place}");
        return;
    }
    let actual = body
        .pointer(&check.at)
        .unwrap_or_else(|| panic!("{place} missing"));
    if let Some(expected) = &check.equals {
        assert_eq!(actual, expected, "{place}");
    }
    if let Some(reference) = &check.same_as {
        let (source, pointer) = reference.split_once(':').unwrap();
        assert_eq!(Some(actual), responses[source].pointer(pointer), "{place}");
    }
    if let Some(expected) = &check.instant {
        assert_eq!(
            moment(actual.as_str().unwrap()),
            moment(expected),
            "{place}"
        );
    }
    if let Some(length) = check.length {
        let count = actual
            .as_array()
            .map(Vec::len)
            .or_else(|| actual.as_object().map(serde_json::Map::len));
        assert_eq!(count, Some(length), "{place}");
    }
}

fn http_request(request: ScenarioRequest) -> Request<Body> {
    let mut builder = Request::builder()
        .method(request.method.as_str())
        .uri(&request.path);
    let has_type = request
        .headers
        .keys()
        .any(|name| name.eq_ignore_ascii_case("content-type"));
    for (name, value) in &request.headers {
        builder = builder.header(name, value);
    }
    let body = match (request.json, request.raw) {
        (Some(json), None) => Some(serde_json::to_vec(&json).unwrap()),
        (None, Some(raw)) => Some(raw.into_bytes()),
        (None, None) => None,
        _ => panic!("a request has either json or raw content"),
    };
    match body {
        Some(bytes) => {
            if !has_type {
                builder = builder.header("content-type", "application/json");
            }
            builder.body(Body::from(bytes)).unwrap()
        }
        None => builder.body(Body::empty()).unwrap(),
    }
}

pub async fn run(name: &str) {
    let scenario = load(name);
    let server = TestServer::start(moment(&scenario.clock));
    if !scenario.seed_sql.is_empty() {
        let connection = rusqlite::Connection::open(&server.database_path).unwrap();
        connection
            .execute_batch(&scenario.seed_sql.join("\n"))
            .unwrap();
    }
    let mut captures = BTreeMap::new();
    let mut responses = BTreeMap::new();
    for step in &scenario.steps {
        if let Some(now) = &step.clock {
            server.clock.set(moment(now));
        }
        let request: ScenarioRequest =
            serde_json::from_value(substitute(&expand(&step.request), &captures)).unwrap();
        let expect: Expectation =
            serde_json::from_value(substitute(&expand(&step.expect), &captures)).unwrap();
        let response = server
            .router
            .clone()
            .oneshot(http_request(request))
            .await
            .unwrap();
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let content = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            status,
            expect.status,
            "{name} {}: {}",
            step.id,
            String::from_utf8_lossy(&content)
        );
        for (header, value) in &expect.headers {
            assert_eq!(
                headers.get(header).and_then(|v| v.to_str().ok()),
                Some(value.as_str()),
                "{} {header}",
                step.id
            );
        }
        if let Some(bytes) = &expect.bytes {
            assert_eq!(
                content.as_ref(),
                scenario_bytes(bytes).as_slice(),
                "{}",
                step.id
            );
            continue;
        }
        let body: Value = if content.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&content).unwrap()
        };
        for item in &expect.json {
            check(&step.id, &body, item, &responses);
        }
        for (capture, pointer) in &step.capture {
            captures.insert(capture.clone(), body.pointer(pointer).unwrap().clone());
        }
        responses.insert(step.id.clone(), body);
    }
}
