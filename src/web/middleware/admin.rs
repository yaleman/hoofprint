// Middleware to add cache headers to static assets and manifest
use crate::{constants::GROUP_ADMIN, prelude::*};

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub(crate) async fn ensure_admin(
    state: State<AppState>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let auth_user = match state.get_authenticated_user(&session).await {
        Ok(val) => val,
        Err(err) => {
            tracing::error!(error=?err, "Failed to get authenticated user in admin middleware");
            return HoofprintError::Unauthorized.into_response();
        }
    };

    if !auth_user.groups.contains(&GROUP_ADMIN.to_string()) {
        return HoofprintError::Unauthorized.into_response();
    }

    next.run(request).await
}

#[tokio::test]
async fn test_admin_unauth() {
    let (server, _db) = crate::tests::setup_test_server().await;

    let response = server.get(Urls::AdminDashboard.as_ref()).await;
    assert_eq!(response.status_code(), axum::http::StatusCode::FORBIDDEN);
}
