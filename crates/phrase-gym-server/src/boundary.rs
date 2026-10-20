use axum::extract::ConnectInfo;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::{CONTENT_LENGTH, HOST, ORIGIN};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http_body_util::{BodyExt, LengthLimitError, Limited};

use crate::error::ApiError;
use crate::settings::ServerSettings;

pub(crate) const MAX_REQUEST_BYTES: usize = 24 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct Boundary {
    allowed_hosts: Vec<String>,
    allowed_origins: Vec<String>,
}

impl Boundary {
    pub(crate) fn new(settings: &ServerSettings) -> Arc<Self> {
        Arc::new(Self {
            allowed_hosts: settings.allowed_hosts.clone(),
            allowed_origins: settings.allowed_origins.clone(),
        })
    }

    fn host(request: &Request) -> &str {
        request
            .headers()
            .get(HOST)
            .and_then(|value| value.to_str().ok())
            .or_else(|| {
                request
                    .uri()
                    .authority()
                    .map(axum::http::uri::Authority::as_str)
            })
            .unwrap_or_default()
    }

    fn trusted_host(&self, request: &Request) -> bool {
        let name = Self::host(request).split(':').next().unwrap_or_default();
        self.allowed_hosts.iter().any(|host| host == name)
    }

    // Same-origin and allowlisted origins are not authentication; they only stop browsers from
    // submitting mutations on behalf of another site.
    fn cross_origin(&self, request: &Request) -> bool {
        if matches!(
            *request.method(),
            Method::GET | Method::HEAD | Method::OPTIONS
        ) {
            return false;
        }
        let headers = request.headers();
        if headers
            .get("sec-fetch-site")
            .is_some_and(|value| value.as_bytes() == b"cross-site")
        {
            return true;
        }
        let origin = headers
            .get(ORIGIN)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        if origin.is_empty() {
            return false;
        }
        let (scheme, rest) = origin.split_once("://").unwrap_or(("", origin));
        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        let same_origin = authority == Self::host(request) && scheme == "http";
        !same_origin && !self.allowed_origins.iter().any(|allowed| allowed == origin)
    }
}

fn declared_too_large(headers: &HeaderMap) -> bool {
    headers.get(CONTENT_LENGTH).is_some_and(|value| {
        value
            .to_str()
            .ok()
            .and_then(|text| text.parse::<u64>().ok())
            .is_none_or(|length| length > MAX_REQUEST_BYTES as u64)
    })
}

fn too_large() -> Response {
    ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "Request exceeds 24 MiB").into_response()
}

// The whole body is read under a counted limit before routing, so a dishonest Content-Length
// cannot exceed it and a client that disconnects mid-upload never reaches a handler.
pub(crate) async fn enforce(
    State(boundary): State<Arc<Boundary>>,
    request: Request,
    next: Next,
) -> Response {
    // The listener is always loopback. Never accept forwarded addresses as socket identity.
    let local = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .is_some_and(|peer| peer.0.ip().is_loopback());
    if !local {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            "Use the direct local Server connection",
        )
        .into_response();
    }
    if !boundary.trusted_host(&request) {
        return (StatusCode::BAD_REQUEST, "Invalid host header").into_response();
    }
    if boundary.cross_origin(&request) {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            "Cross-origin mutations are not allowed",
        )
        .into_response();
    }
    if declared_too_large(request.headers()) {
        return too_large();
    }
    next.run(request).await
}

pub(crate) async fn bounded_body(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    match Limited::new(body, MAX_REQUEST_BYTES).collect().await {
        Ok(collected) => {
            next.run(Request::from_parts(parts, Body::from(collected.to_bytes())))
                .await
        }
        Err(error) if error.downcast_ref::<LengthLimitError>().is_some() => too_large(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
