use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

// Middleware to log things the way I like
pub(crate) async fn logger(request: Request, next: Next) -> Response {
    let req_uri = request.uri().clone();
    let method = request.method().clone();

    let response = next.run(request).await;

    info!(uri = %req_uri.path(), status = %response.status().as_u16(), method=%method, "request");

    response
}
