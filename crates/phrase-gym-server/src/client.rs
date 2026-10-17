use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use crate::error::ApiError;

const MISSING_CLIENT: &str = "Build the client and pass --client-dir frontend/dist";

#[derive(Debug)]
struct ClientFiles {
    root: PathBuf,
}

pub(crate) fn routes(client_dir: Option<&Path>) -> Router {
    match client_dir.filter(|directory| directory.join("index.html").is_file()) {
        Some(directory) => Router::new()
            .route("/", get(index))
            .route("/index.html", get(index))
            .route("/learn", get(index))
            .route("/learn/{*rest}", get(index))
            .route("/icon.svg", get(icon))
            .route("/assets/{*path}", get(asset))
            .with_state(Arc::new(ClientFiles {
                root: directory.to_owned(),
            })),
        None => Router::new().route(
            "/",
            get(|| async { ApiError::new(StatusCode::SERVICE_UNAVAILABLE, MISSING_CLIENT) }),
        ),
    }
}

async fn index(State(files): State<Arc<ClientFiles>>) -> Response {
    serve(files.root.join("index.html")).await
}

async fn icon(State(files): State<Arc<ClientFiles>>) -> Response {
    serve(files.root.join("icon.svg")).await
}

async fn asset(State(files): State<Arc<ClientFiles>>, UrlPath(path): UrlPath<String>) -> Response {
    if !path.split('/').all(safe_segment) {
        return not_found();
    }
    serve(files.root.join("assets").join(path)).await
}

// Only plain names reach the filesystem, so no request can climb out of the assets directory.
fn safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('.')
        && segment
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-~+@".contains(character))
}

async fn serve(path: PathBuf) -> Response {
    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let mut response = bytes.into_response();
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static(content_type(&path)));
            response
        }
        Err(_) => not_found(),
    }
}

pub(crate) fn not_found() -> Response {
    ApiError::new(StatusCode::NOT_FOUND, "Not Found").into_response()
}

fn content_type(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "json" | "map" => "application/json",
        "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        _ => "application/octet-stream",
    }
}
