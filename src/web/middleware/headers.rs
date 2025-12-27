use axum::{
    extract::{Request, State},
    http::{
        HeaderValue,
        header::{
            // ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            CACHE_CONTROL,
        },
    },
    middleware::Next,
    response::Response,
};

use crate::{prelude::Urls, web::AppState};

// Middleware to add cache headers to static assets and manifest
pub(crate) async fn apply_headers(
    state: State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let request_uri = request.uri().clone();
    let server_uri: String = state.base_url.clone();
    let path = request_uri.path();
    let is_static = path.starts_with(Urls::Static.as_ref());
    let needs_cache = is_static || path == Urls::Manifest.as_ref();

    let mut response = next.run(request).await;

    if needs_cache {
        response.headers_mut().insert(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        );
    }

    // Allow cross-origin access to static assets (e.g., .wasm) when embedded or fetched.
    // This is safe for static files and avoids fetch() CORS errors for WASM loaders.
    let headers = response.headers_mut();
    // Allow any origin to fetch static assets (no credentials involved).
    if let Ok(header) = HeaderValue::from_str(&server_uri) {
        headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, header);
    }
    headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, HEAD, OPTIONS"),
    );
    // headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));

    response
}
