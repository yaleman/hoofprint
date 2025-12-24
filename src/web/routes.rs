use crate::prelude::*;
use axum::routing::{get, post};

use super::state::AppState;
use super::views;

/// Creates the application router with all routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(views::homepage))
        .route("/view/{code}", get(views::view_code))
        .route(
            "/edit/{code}",
            get(views::edit_code_get).post(views::edit_code_post),
        )
        .route("/manifest.json", get(super::manifest::manifest))
        .route(
            "/create",
            get(views::create_code_get).post(views::create_code_post),
        )
        .route("/delete/{code}", post(views::code_delete))
        .route("/scan", get(views::scan_get).post(views::scan_post))
        .route(
            "/login",
            get(super::auth::get_login).post(super::auth::post_login),
        )
        .route(
            "/logout",
            post(super::auth::logout).get(super::auth::logout),
        )
}
