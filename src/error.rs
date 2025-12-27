use std::{sync::Arc, time::SystemTimeError};

use axum::{
    body::Body,
    http::{
        HeaderValue, Response, StatusCode,
        header::{LOCATION, ToStrError},
    },
    response::IntoResponse,
};
use rustls::crypto::CryptoProvider;
use tokio::task::JoinError;
use tracing::error;

#[derive(Debug)]
pub enum HoofprintError {
    Template(askama::Error),
    Database(String),
    NotFound(String),
    ValidationError(Vec<String>),
    Authentication,
    Unauthorized,
    InvalidCodeType(String),
    InvalidSite,
    InvalidBaseUrl(String),
    InternalError(String),
    NeedToLogin,
}

impl std::fmt::Display for HoofprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoofprintError::Template(err) => write!(f, "Template Error: {}", err),
            HoofprintError::Database(err) => write!(f, "Database Error: {}", err),
            HoofprintError::NotFound(entity) => write!(f, "Not Found: {}", entity),
            HoofprintError::ValidationError(errors) => {
                write!(f, "Validation Error: {}", errors.join(", "))
            }
            HoofprintError::Authentication => write!(f, "Authentication Failed"),
            HoofprintError::Unauthorized => write!(f, "Unauthorized Access"),
            HoofprintError::InvalidCodeType(code_type) => {
                write!(f, "Invalid Code Type: {}", code_type)
            }
            HoofprintError::InvalidSite => write!(f, "Invalid Site"),
            HoofprintError::InvalidBaseUrl(url) => write!(f, "Invalid Base URL: {}", url),
            HoofprintError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            HoofprintError::NeedToLogin => write!(f, "Need to Login"),
        }
    }
}

impl std::error::Error for HoofprintError {}

impl From<ToStrError> for HoofprintError {
    fn from(err: ToStrError) -> Self {
        HoofprintError::InternalError(err.to_string())
    }
}

impl From<std::io::Error> for HoofprintError {
    fn from(err: std::io::Error) -> Self {
        error!("IO error: {}", err);
        HoofprintError::InternalError("IO Error, check the logs!".to_string())
    }
}

impl From<JoinError> for HoofprintError {
    fn from(err: JoinError) -> Self {
        error!("Join error: {}", err);
        HoofprintError::InternalError("Join Error, check the logs!".to_string())
    }
}

impl From<tower_sessions::session_store::Error> for HoofprintError {
    fn from(err: tower_sessions::session_store::Error) -> Self {
        error!("Session store error: {}", err);
        HoofprintError::InternalError("Session Store Error, check the logs!".to_string())
    }
}

impl From<SystemTimeError> for HoofprintError {
    fn from(err: SystemTimeError) -> Self {
        error!("System time error: {}", err);
        HoofprintError::InternalError("System Time Error, check the logs!".to_string())
    }
}

impl From<argon2::password_hash::Error> for HoofprintError {
    fn from(err: argon2::password_hash::Error) -> Self {
        error!("Password hashing error: {}", err);
        HoofprintError::InternalError("Password Hashing Error, check the logs!".to_string())
    }
}

impl From<tower_sessions::session::Error> for HoofprintError {
    fn from(err: tower_sessions::session::Error) -> Self {
        error!("Session error: {}", err);
        HoofprintError::InternalError("Session Error, check the logs!".to_string())
    }
}

impl From<askama::Error> for HoofprintError {
    fn from(err: askama::Error) -> Self {
        HoofprintError::Template(err)
    }
}

impl From<sea_orm::SqlxError> for HoofprintError {
    fn from(err: sea_orm::SqlxError) -> Self {
        HoofprintError::Database(err.to_string())
    }
}

impl From<sea_orm::DbErr> for HoofprintError {
    fn from(err: sea_orm::DbErr) -> Self {
        HoofprintError::Database(err.to_string())
    }
}

impl From<url::ParseError> for HoofprintError {
    fn from(err: url::ParseError) -> Self {
        HoofprintError::InternalError(format!("Failed to parse URL: {}", err))
    }
}

impl From<Arc<CryptoProvider>> for HoofprintError {
    fn from(err: Arc<CryptoProvider>) -> Self {
        HoofprintError::InternalError(format!("Crypto Error: {:?}", err))
    }
}

impl From<std::net::AddrParseError> for HoofprintError {
    fn from(err: std::net::AddrParseError) -> Self {
        HoofprintError::InternalError(format!("Address Parse Error: {}", err))
    }
}

impl From<serde_json::error::Error> for HoofprintError {
    fn from(err: serde_json::error::Error) -> Self {
        HoofprintError::InternalError(format!("JSON Error: {}", err))
    }
}

impl IntoResponse for HoofprintError {
    fn into_response(self) -> Response<Body> {
        // Log the error for debugging
        error!("Error occurred: {:?}", self);

        match self {
            HoofprintError::Template(err) => {
                let body = format!("Template Error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            HoofprintError::InvalidBaseUrl(err) => {
                let body = format!("Invalid Base URL: {}", err);
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            HoofprintError::Database(err) => {
                let body = format!("Database Error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            HoofprintError::NotFound(entity) => {
                let body = format!("Not Found: {}", entity);
                (StatusCode::NOT_FOUND, body).into_response()
            }
            HoofprintError::ValidationError(errors) => {
                let body = format!("Validation Errors:\n{}", errors.join("\n"));
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            HoofprintError::Authentication => {
                let body = "Authentication Failed";
                (StatusCode::UNAUTHORIZED, body).into_response()
            }
            HoofprintError::Unauthorized => {
                let body = "Unauthorized Access";
                (StatusCode::FORBIDDEN, body).into_response()
            }
            HoofprintError::InvalidSite => {
                let body = "Invalid Site";
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            HoofprintError::InvalidCodeType(code_type) => {
                let body = format!("Invalid Code Type: {}", code_type);
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            HoofprintError::InternalError(msg) => {
                let body = format!("Internal Error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            HoofprintError::NeedToLogin => {
                // redirect the user to the login page
                (
                    StatusCode::SEE_OTHER,
                    [(LOCATION, HeaderValue::from_static("/login"))],
                )
                    .into_response()
            }
        }
    }
}
