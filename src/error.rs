use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

#[derive(Debug)]
pub enum HoofprintError {
    Template(askama::Error),
    // Other error variants can be added here
}

impl std::fmt::Display for HoofprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoofprintError::Template(err) => write!(f, "Template Error: {}", err),
            // Handle other error variants here
        }
    }
}

impl From<askama::Error> for HoofprintError {
    fn from(err: askama::Error) -> Self {
        HoofprintError::Template(err)
    }
}

impl IntoResponse for HoofprintError {
    fn into_response(self) -> Response<Body> {
        match self {
            HoofprintError::Template(err) => {
                let body = format!("Template Error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            } // Handle other error variants here
        }
    }
}
