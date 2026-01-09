use std::collections::HashMap;

use axum::{Form, extract::Query};
use secret_string::SecretString;

use crate::{db::entities::user, prelude::*};

#[derive(Serialize, Deserialize, Template, WebTemplate)]
#[template(path = "register.html")]
pub(crate) struct RegisterPage {
    pub(crate) error: Option<String>,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) password: SecretString<String>,
}

pub(crate) async fn get_register(
    Query(query): Query<HashMap<String, String>>,
) -> Result<RegisterPage, HoofprintError> {
    let register_page = RegisterPage {
        name: query.get("name").cloned().unwrap_or_default(),
        email: query.get("email").cloned().unwrap_or_default(),
        error: query.get("error").cloned(),
        password: SecretString::new("".to_string()),
    };

    Ok(register_page)
}

pub(crate) async fn post_register(
    State(app_state): State<AppState>,
    session: Session,
    Form(form): Form<RegisterPage>,
) -> Result<Redirect, HoofprintError> {
    if app_state.get_authenticated_user(&session).await.is_ok() {
        return Ok(Redirect::to(Urls::Home.as_ref()));
    };

    if user::Entity::find()
        .filter(user::Column::Email.eq(form.email.clone()))
        .one(&app_state.db)
        .await?
        .is_some()
    {
        let redirect_url = format!(
            "{}?error=User with that email already exists!&email={}&name={}",
            Urls::Register.as_ref(),
            form.email,
            form.name
        );
        return Ok(Redirect::to(&redirect_url));
    }

    let mut errors = Vec::new();
    if form.email.trim().is_empty() {
        errors.push("Email is required".to_string());
    }
    if form.password.value().trim().is_empty() {
        errors.push("Password is required".to_string());
    }

    if !errors.is_empty() {
        let error_query = format!("error={}", errors.join(", "));
        let redirect_url = format!("{}?{}", Urls::Register.as_ref(), error_query);
        return Ok(Redirect::to(&redirect_url));
    }

    if let Err(err) = user::Model::create_new(
        app_state.db.clone(),
        &form.email,
        &form.name,
        Some(form.password.value()),
    )
    .await
    {
        error!(error=?err, email=%form.email, "Failed to create new user");
        let redirect_url = format!(
            "{}?error=Failed to create new user!&email={}&name={}",
            Urls::Register.as_ref(),
            form.email,
            form.name
        );
        return Ok(Redirect::to(&redirect_url));
    };
    info!(email=%form.email, "Created new user account");

    let redirect_url = format!(
        "{}?success=Account created successfully! Please log in.&email={}",
        Urls::Login.as_ref(),
        form.email
    );

    Ok(Redirect::to(&redirect_url))
}
