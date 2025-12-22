use crate::prelude::*;

use super::state::AppState;
use super::views;

/// Creates the application router with all routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(views::homepage))
        .route("/view/{code}", get(views::view_code))
}
