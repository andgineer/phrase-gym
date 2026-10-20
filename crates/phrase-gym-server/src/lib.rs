mod actor;
mod boundary;
mod client;
mod error;
mod extract;
pub mod openapi;
mod range;
mod routes;
mod settings;
mod state;
#[cfg(feature = "test-support")]
mod test_support;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::middleware;
use axum::response::Response;

pub use settings::ServerSettings;
pub use state::AppState;

// Every response is uncached: practice is online-only and no client asset may outlive a deployment.
async fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

pub fn router(state: AppState, settings: &ServerSettings) -> Router {
    #[cfg(feature = "test-support")]
    let browser = state.browser.clone();
    let api = routes::api().with_state(state.clone());
    #[cfg(feature = "test-support")]
    let api =
        api.merge(test_support::routes().with_state(state))
            .layer(middleware::from_fn_with_state(
                browser,
                test_support::intercept,
            ));
    Router::new()
        .merge(api)
        .merge(client::routes(settings.client_dir.as_deref()))
        .fallback(|| async { client::not_found() })
        .layer(axum::Extension(actor::Actor::default()))
        .layer(middleware::from_fn(boundary::bounded_body))
        .layer(DefaultBodyLimit::max(boundary::MAX_REQUEST_BYTES))
        .layer(middleware::from_fn_with_state(
            boundary::Boundary::new(settings),
            boundary::enforce,
        ))
        .layer(middleware::map_response(no_store))
}
