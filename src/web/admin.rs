//! Admin UI handlers
use sea_orm::{ActiveModelTrait, IntoActiveModel, Order, QueryOrder};

use crate::{constants::PASSWORD_DEFAULT_LENGTH, get_random_password, prelude::*};

#[derive(Template, WebTemplate)]
#[template(path = "admin_dashboard.html")]
pub(crate) struct AdminDashboardPage {
    pub user_email: String,
    pub user_display_name: String,
    pub users: Vec<user::Model>,
}

pub(crate) async fn dashboard_get(
    app_state: State<AppState>,
    session: Session,
) -> Result<AdminDashboardPage, HoofprintError> {
    let auth_user = app_state.get_authenticated_user(&session).await?;

    let users = user::Entity::find()
        .order_by(user::Column::Email, Order::Asc)
        .all(&app_state.db)
        .await?;

    let dashboard_page = AdminDashboardPage {
        user_email: auth_user.email,
        user_display_name: auth_user.display_name,
        users,
    };

    Ok(dashboard_page)
}

#[derive(Deserialize)]
pub(crate) struct PwUserQuery {
    pub user_id: Uuid,
}

#[derive(Template, WebTemplate)]
#[template(path = "admin_password_reset_confirm.html")]
pub(crate) struct PasswordResetConfirm {
    pub user_email: String,
    pub user_display_name: String,
    pub user_id: Uuid,
    pub csrf_token: String,
}

pub(crate) async fn password_reset_get(
    Query(query): Query<PwUserQuery>,
    app_state: State<AppState>,
    session: Session,
) -> Result<PasswordResetConfirm, HoofprintError> {
    let _auth_user = app_state.get_authenticated_user(&session).await?;

    // get the user to ensure they exist
    let target_user = user::Entity::find_by_id(query.user_id)
        .one(&app_state.db)
        .await?;

    let csrf_token = Uuid::now_v7().to_string();
    session.insert("csrf_token", csrf_token.clone()).await?;

    match target_user {
        Some(user) => {
            let reset_page = PasswordResetConfirm {
                user_email: user.email,
                user_display_name: user.display_name,
                user_id: user.id,
                csrf_token,
            };
            Ok(reset_page)
        }
        None => Err(HoofprintError::NotFound("User not found".into())),
    }
}

#[derive(Deserialize)]
pub(crate) struct AdminPwResetForm {
    pub user_id: Uuid,
    pub csrf_token: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "admin_password_reset_page.html")]
pub(crate) struct AdminPasswordResetComplete {
    pub user_email: String,
    pub user_display_name: String,
    pub user_id: Uuid,
    pub new_password: String,
}

pub(crate) async fn password_reset_post(
    State(app_state): State<AppState>,
    session: Session,
    Form(form): Form<AdminPwResetForm>,
) -> Result<AdminPasswordResetComplete, HoofprintError> {
    let auth_user = app_state.get_authenticated_user(&session).await?;

    match session.remove_value("csrf_token").await? {
        Some(token) => {
            if token != form.csrf_token {
                return Err(HoofprintError::InvalidCsrfToken);
            }
        }
        None => return Err(HoofprintError::MissingCsrfToken)?,
    };

    // ensure the user exists
    let target_user = user::Entity::find_by_id(form.user_id)
        .one(&app_state.db)
        .await?;
    match target_user {
        Some(user) => {
            // reset the user's password to a default value
            let password = get_random_password(PASSWORD_DEFAULT_LENGTH);
            let hashed_password = crate::password::hash_password(&password)?;
            let mut user_active = user.clone().into_active_model();
            user_active.password.set_if_not_equals(hashed_password);
            let updated_user = user_active.update(&app_state.db).await?;
            info!(admin_user = %auth_user.email, user_email = %user.email, "Admin reset password for user");
            Ok(AdminPasswordResetComplete {
                user_email: updated_user.email,
                user_display_name: updated_user.display_name,
                user_id: updated_user.id,
                new_password: password,
            })
        }
        None => Err(HoofprintError::NotFound("User not found".into())),
    }
}
