use crate::prelude::*;

use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};

use super::state::AppState;
use super::views;

/// Creates the application router with all routes
pub fn routes(state: &AppState) -> Router<AppState> {
    // TODO: add authentication layer/middleware
    let requires_admin = Router::new()
        .route(
            Urls::AdminDashboard.as_ref(),
            get(super::admin::dashboard_get),
        )
        .route(
            Urls::AdminPasswordReset.as_ref(),
            get(super::admin::password_reset_get).post(super::admin::password_reset_post),
        )
        .layer(from_fn_with_state(
            state.clone(),
            super::middleware::admin::ensure_admin,
        ));

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

    Router::new()
        .merge(requires_admin)
        .merge(requires_auth)
        .route(
            Urls::Register.as_ref(),
            get(super::registration::get_register).post(super::registration::post_register),
        )
        .route(
            Urls::Login.as_ref(),
            get(super::auth::get_login).post(super::auth::post_login),
        )
        .route(
            Urls::CspReportOnly.as_ref(),
            post(super::views::csp_report_only),
        )
        .route(Urls::HealthCheck.as_ref(), get(super::views::health_check))
}

#[tokio::test]
async fn test_get_router() {
    let router = routes(&AppState::test().await);
    dbg!(&router);
    let _test = router.without_v07_checks();
}
