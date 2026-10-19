use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Router};

use crate::authority::{Capability, Credential, Grant};
use crate::settings::ServerSettings;

pub(crate) fn routes(settings: &ServerSettings) -> Router {
    Router::new()
        .route("/login", get(|| async { Html(LOGIN) }))
        .merge(
            Router::new()
                .route("/api/v1/auth/session", post(session))
                .route_layer(middleware::from_fn_with_state(
                    Grant {
                        authority: settings.authority.clone(),
                        any: vec![Capability::Learner],
                    },
                    crate::authority::enforce,
                )),
        )
}

async fn session(Extension(credential): Extension<Credential>, headers: HeaderMap) -> Response {
    if credential.capabilities != [Capability::Learner] {
        return StatusCode::FORBIDDEN.into_response();
    }
    let secure = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| origin.starts_with("https://"));
    let cookie = format!(
        "phrase_gym_learner={}; Path=/; HttpOnly; SameSite=Strict{}",
        credential.token,
        if secure { "; Secure" } else { "" }
    );
    (StatusCode::NO_CONTENT, [(header::SET_COOKIE, cookie)]).into_response()
}

const LOGIN: &str = r#"<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Phrase Gym login</title>
<main><h1>Phrase Gym</h1><form><label>Learner access token <input name="token" type="password" autocomplete="off" required></label><button>Sign in</button><p role="status"></p></form></main>
<script>
document.querySelector('form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const input = document.querySelector('input');
  try {
    const response = await fetch('/api/v1/auth/session', { method: 'POST', headers: { Authorization: `Bearer ${input.value}` } });
    input.value = '';
    if (!response.ok) throw new Error('Sign-in failed. Check the learner token.');
    location.replace('/');
  } catch (error) { document.querySelector('[role=status]').textContent = error.message; }
});
</script></html>"#;
