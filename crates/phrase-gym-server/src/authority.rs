use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Learner,
    MaterialSubmit,
    Worker,
    ProfileRead,
    ProfileUpdate,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credential {
    pub actor: String,
    pub token: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Clone, Default)]
pub struct Authority {
    credentials: Vec<Credential>,
}

impl Authority {
    pub fn new(credentials: Vec<Credential>) -> Result<Self, String> {
        for (index, credential) in credentials.iter().enumerate() {
            if credential.actor.trim().is_empty()
                || credential.actor.len() > 200
                || credential.token.len() < 32
                || credential.token.len() > 256
                || !credential
                    .token
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
                || credential.capabilities.is_empty()
                || credentials[..index]
                    .iter()
                    .any(|other| other.token == credential.token || other.actor == credential.actor)
            {
                return Err("Credentials require unique actors/tokens, 32–256 URL-safe token characters and explicit capabilities".into());
            }
        }
        Ok(Self { credentials })
    }

    pub(crate) fn authenticate(&self, request: &Request) -> Option<Credential> {
        let bearer = request
            .headers()
            .get("authorization")
            .and_then(|value| value.to_str().ok());
        let token = if let Some(bearer) = bearer {
            bearer.strip_prefix("Bearer ")?
        } else {
            request
                .headers()
                .get("cookie")
                .and_then(|value| value.to_str().ok())?
                .split(';')
                .find_map(|part| part.trim().strip_prefix("phrase_gym_learner="))?
        };
        self.credentials
            .iter()
            .find(|credential| same_token(&credential.token, token))
            .cloned()
    }
}

// Keep token-byte comparison independent of the position of a mismatch.
fn same_token(expected: &str, supplied: &str) -> bool {
    if expected.len() != supplied.len() {
        return false;
    }
    expected
        .bytes()
        .zip(supplied.bytes())
        .fold(0_u8, |difference, (left, right)| {
            std::hint::black_box(difference | (left ^ right))
        })
        == 0
}

#[derive(Clone)]
pub(crate) struct Grant {
    pub authority: Arc<Authority>,
    pub any: Vec<Capability>,
}

pub(crate) async fn enforce(
    State(grant): State<Grant>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(credential) = grant.authority.authenticate(&request) else {
        return ApiError::new(StatusCode::UNAUTHORIZED, "Authentication required").into_response();
    };
    if !grant
        .any
        .iter()
        .any(|capability| credential.capabilities.contains(capability))
    {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            "Credential lacks the required capability",
        )
        .into_response();
    }
    // Only learner-only credentials may be used as ambient browser cookies.
    if !request.headers().contains_key("authorization")
        && credential.capabilities != [Capability::Learner]
    {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            "API credentials require Bearer authorization",
        )
        .into_response();
    }
    request.extensions_mut().insert(credential);
    crate::boundary::bounded_body(request, next).await
}
