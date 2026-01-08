use std::str::FromStr;

use axum::{
    extract::{Request, State},
    http::{
        HeaderName, HeaderValue,
        header::{
            // ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            CACHE_CONTROL,
            CONTENT_SECURITY_POLICY,
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

    let headers = response.headers_mut();
    if needs_cache {
        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        );
    }
    headers.insert(
        ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Access-Control-Allow-Headers: Content-Type"),
    );

    // TODO: remove the style-src 'unsafe-inline' once <https://github.com/lindell/JsBarcode/pull/474> is merged and released
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
        script-src 'self' 'wasm-unsafe-eval' 'report-sha256'; \
        style-src 'self'; \
        img-src 'self' data:; \
        font-src 'self'; \
        connect-src 'self'; \
        frame-ancestors 'none'; \
        base-uri 'self'; \
        form-action 'self'; \
        report-uri /csp/reportOnly;",
        ),
    );

    // Report target for CSP violations
    #[allow(clippy::expect_used)]
    headers.insert(HeaderName::from_str("Report-To").expect("Invalid header name in code"), HeaderValue::from_static(r#"{"group":"default","max_age":31536000,"endpoints":[{"url":"/csp/reportOnly"}],"include_subdomains":true}"#));

    // For static assets, set CORS headers to allow access from our configured origin
    // This ensures WASM files and other static assets can be loaded correctly
    if is_static {
        if let Ok(header) = HeaderValue::from_str(&server_uri) {
            headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, header);
        }
        headers.insert(
            ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, HEAD, OPTIONS"),
        );
    }

    response
}
