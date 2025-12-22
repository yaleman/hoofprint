use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
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
        }
    }
}

impl std::error::Error for HoofprintError {}

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

impl IntoResponse for HoofprintError {
    fn into_response(self) -> Response<Body> {
        // Log the error for debugging
        error!("Error occurred: {:?}", self);

        match self {
            HoofprintError::Template(err) => {
                let body = format!("Template Error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
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
        }
    }
}
