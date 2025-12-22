//! Authentication module for hoofprint

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::error::HoofprintError;

/// Extractor for authenticated user information
/// Currently hardcoded to return the default admin user for MVP
/// TODO: Implement real authentication (OAuth/OIDC, session management, etc.)
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = HoofprintError;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // TODO: Implement real authentication
        // For MVP, hardcode to default admin user
        Ok(AuthenticatedUser {
            user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                .map_err(|_| HoofprintError::Authentication)?,
        })
    }
}
