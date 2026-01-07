//! Authentication module for hoofprint

use std::collections::HashMap;

use crate::{constants::Urls, db::entities::user, password::verify_password, prelude::*};

use axum::{
    Form,
    extract::Query,
    http::{StatusCode, header::LOCATION},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tower_sessions::Session;

pub(crate) const AUTH_USER_ID: &str = "user_id";

/// Extractor for authenticated user information
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub email: String,
    #[allow(dead_code)]
    pub groups: Vec<String>,
}

impl From<user::Model> for AuthenticatedUser {
    fn from(user: user::Model) -> Self {
        Self {
            user_id: user.id,
            email: user.email.clone(),
            groups: user
                .groups
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "login_form.html")]
pub(crate) struct LoginPage {
    pub email: String,
    pub success: Option<String>,
    pub error: Option<String>,
}

#[instrument(level = "debug", skip_all)]
pub(crate) async fn get_login(
    Query(query): Query<HashMap<String, String>>,
) -> Result<LoginPage, HoofprintError> {
    let login_page = LoginPage {
        email: query.get("email").cloned().unwrap_or_default(),
        error: query.get("error").cloned(),
        success: query.get("success").cloned(),
    };

    Ok(login_page)
}

#[derive(Serialize, Deserialize, Template, WebTemplate)]
#[template(path = "login_form.html")]
pub(crate) struct LoginForm {
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) error: Option<String>,
    pub(crate) success: Option<String>,
}

#[instrument(level="debug", skip(form, app_state, session), fields(email = %form.email))]
pub(crate) async fn post_login(
    app_state: State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Result<axum::response::Response, HoofprintError> {
    if form.password.is_empty() || form.email.is_empty() {
        let login_page = LoginForm {
            email: form.email,
            password: "".to_string(),
            error: Some("Email or password cannot be empty.".to_string()),
            success: None,
        };
        return Ok(login_page.into_response());
    }

    // check if the user exists
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(form.email.clone()))
        .one(&app_state.db)
        .await?;

    match user {
        None => {
            let login_page = LoginForm {
                email: form.email,
                password: "".to_string(),
                error: Some("Invalid email or password.".to_string()),
                success: None,
            };
            return Ok(login_page.into_response());
        }
        Some(user) => {
            // verify the password
            match verify_password(&form.password, &user.password) {
                Err(err) => {
                    error!(error=?err, email=%form.email, "Password verification failed");
                    session.delete().await?;
                    let login_page = LoginForm {
                        email: form.email,
                        password: "".to_string(),
                        error: Some("Invalid email or password.".to_string()),
                        success: None,
                    };
                    return Ok(login_page.into_response());
                }
                Ok(()) => {
                    info!(email=%form.email, "User authenticated successfully");
                    session.insert(AUTH_USER_ID, user.id.to_string()).await?;
                    session.save().await?;
                }
            }
        }
    }

    Ok((StatusCode::SEE_OTHER, [(LOCATION, "/")]).into_response())
}

#[instrument(level="debug",skip_all, fields(user_id = %session.get::<String>(AUTH_USER_ID).await?.unwrap_or("unknown-user".to_string())))]
pub(crate) async fn logout(session: Session) -> Result<axum::response::Response, HoofprintError> {
    let userid: String = session
        .get(AUTH_USER_ID)
        .await?
        .unwrap_or("unknown".to_string());
    session.delete().await?;
    debug!("User {} logged out", userid);
    Ok((StatusCode::SEE_OTHER, [(LOCATION, Urls::Login.as_ref())]).into_response())
}
