use axum::{Router, response::Html, routing::get};
use tracing::instrument;

use super::state::AppState;

/// Creates the application router with all routes
pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(homepage))
}

/// Homepage handler that returns a simple HTML response
#[instrument(level = "info")]
async fn homepage() -> Html<&'static str> {
    Html("hoofprints")
}
