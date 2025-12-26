use crate::prelude::*;
use axum::routing::{get, post};

use super::state::AppState;
use super::views;

/// Creates the application router with all routes
pub fn routes() -> Router<AppState> {
    let requires_auth = Router::new()
        .route(Urls::Home.as_ref(), get(views::homepage))
        .route("/view/{code}", get(views::view_code))
        .route(
            "/edit/{code}",
            get(views::edit_code_get).post(views::edit_code_post),
        )
        .route(Urls::Manifest.as_ref(), get(super::manifest::manifest))
        .route(
            Urls::Create.as_ref(),
            get(views::create_code_get).post(views::create_code_post),
        )
        .route("/delete/{code}", post(views::code_delete))
        .route(
            Urls::Scan.as_ref(),
            get(views::scan_get).post(views::scan_post),
        )
        .route(
            Urls::Logout.as_ref(),
            post(super::auth::logout).get(super::auth::logout),
        );

    Router::new().merge(requires_auth).route(
        Urls::Login.as_ref(),
        get(super::auth::get_login).post(super::auth::post_login),
    )
}
