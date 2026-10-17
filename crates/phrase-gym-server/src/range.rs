use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_RANGE, CONTENT_TYPE, ETAG};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

fn parse(requested: &str, length: usize) -> Option<(usize, usize)> {
    let bounds = requested.strip_prefix("bytes=")?;
    if bounds.contains(',') || length == 0 {
        return None;
    }
    let (start, end) = bounds.split_once('-')?;
    let (start, end) = match (start.trim(), end.trim()) {
        ("", suffix) => (length.saturating_sub(suffix.parse().ok()?), length - 1),
        (start, "") => (start.parse().ok()?, length - 1),
        (start, end) => (
            start.parse().ok()?,
            end.parse::<usize>().ok()?.min(length - 1),
        ),
    };
    (start <= end && start < length).then_some((start, end))
}

// Original bytes with a strong validator; a single byte range yields 206 and anything else
// unsatisfiable yields 416, as media elements expect when seeking.
pub(crate) fn bytes_response(
    bytes: Vec<u8>,
    mime_type: &str,
    sha256: &str,
    headers: &HeaderMap,
) -> Response {
    let length = bytes.len();
    let requested = headers.get("range").and_then(|value| value.to_str().ok());
    let (mut response, content_range) = match requested.map(|range| parse(range, length)) {
        None => (bytes.into_response(), None),
        Some(Some((start, end))) => (
            (
                StatusCode::PARTIAL_CONTENT,
                Body::from(bytes[start..=end].to_vec()),
            )
                .into_response(),
            Some(format!("bytes {start}-{end}/{length}")),
        ),
        Some(None) => (
            StatusCode::RANGE_NOT_SATISFIABLE.into_response(),
            Some(format!("bytes */{length}")),
        ),
    };
    if let Some(value) = content_range.and_then(|text| HeaderValue::from_str(&text).ok()) {
        response.headers_mut().insert(CONTENT_RANGE, value);
    }
    let headers = response.headers_mut();
    headers.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    if let Ok(value) = HeaderValue::from_str(&format!("\"{sha256}\"")) {
        headers.insert(ETAG, value);
    }
    if let Ok(value) = HeaderValue::from_str(mime_type) {
        headers.insert(CONTENT_TYPE, value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn single_ranges_are_clamped_and_others_refused() {
        assert_eq!(parse("bytes=0-43", 100), Some((0, 43)));
        assert_eq!(parse("bytes=90-200", 100), Some((90, 99)));
        assert_eq!(parse("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse("bytes=50-", 100), Some((50, 99)));
        for refused in [
            "bytes=100-",
            "bytes=5-1",
            "bytes=0-1,4-5",
            "items=0-1",
            "bytes=x-1",
        ] {
            assert_eq!(parse(refused, 100), None, "{refused}");
        }
        assert_eq!(parse("bytes=0-0", 0), None);
    }
}
