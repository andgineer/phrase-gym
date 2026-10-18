use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::{Controls, wait};
use crate::boundary::MAX_REQUEST_BYTES;
use crate::error::ApiError;

// Faults wrap real handlers: acknowledged writes commit before their response is held or lost.
pub(crate) async fn intercept(
    State(controls): State<Arc<Controls>>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_owned();
    let recording = request.method() == Method::POST && path == "/api/v1/recordings";
    let events = request.method() == Method::POST
        && path.starts_with("/api/v1/sessions/")
        && path.ends_with("/events");
    let (pending, losing, recording_gate, session_gate) = {
        let mut faults = controls.lock();
        let session = if request.method() == Method::GET {
            faults
                .session
                .as_mut()
                .filter(|(id, _)| id.is_some_and(|id| path == format!("/api/v1/sessions/{id}")))
                .map(|(id, gate)| {
                    *id = None;
                    gate.subscribe()
                })
        } else {
            None
        };
        (
            faults.pending,
            faults.lose_rating,
            faults
                .recording
                .as_ref()
                .map(tokio::sync::watch::Sender::subscribe),
            session,
        )
    };
    if pending && (recording || events) {
        return ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Pending delivery unavailable",
        )
        .into_response();
    }
    let mut rate = false;
    if losing && events {
        let (parts, body) = request.into_parts();
        let Ok(bytes) = to_bytes(body, MAX_REQUEST_BYTES).await else {
            return ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "Request too large")
                .into_response();
        };
        if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            rate = payload["events"]
                .as_array()
                .is_some_and(|events| events.iter().any(|event| event["kind"] == "rate"));
        }
        request = Request::from_parts(parts, Body::from(bytes));
    }
    let response = next.run(request).await;
    if response.status() == StatusCode::OK {
        if let Some(gate) = session_gate {
            controls.lock().session_held = true;
            wait(gate).await;
            controls.lock().session_held = false;
        }
        if recording && let Some(gate) = recording_gate {
            wait(gate).await;
        }
        if rate {
            return ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Rating acknowledgement lost",
            )
            .into_response();
        }
    }
    response
}
