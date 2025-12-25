use crate::{db::entities::user, prelude::*, web::auth::AUTH_USER_ID};

/// Application state shared across all web handlers
#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: SendableConfig,
}

impl AppState {
    pub fn new(db: DatabaseConnection, config: SendableConfig) -> Self {
        Self { db, config }
    }

    pub(crate) async fn base_url(&self) -> String {
        let config = self.config.read().await;
        let scheme = if config.tls_certificate.is_some() && config.tls_key.is_some() {
            "https"
        } else {
            "http"
        };
        let port = match config.port.get() {
            80 | 443 => "".to_string(),
            other => format!(":{}", other),
        };
        format!("{}://{}{}", scheme, config.frontend_hostname, port)
    }

    pub(crate) async fn get_authenticated_user(
        &self,
        session: &tower_sessions::Session,
    ) -> Result<crate::web::auth::AuthenticatedUser, HoofprintError> {
        if let Some(user_id_str) = session.get::<String>(AUTH_USER_ID).await? {
            let user_id = Uuid::parse_str(&user_id_str).map_err(|_| {
                error!(user_id=?user_id_str, "Invalid user ID in session");
                HoofprintError::NeedToLogin
            })?;
            let user = user::Entity::find_by_id(user_id)
                .one(&self.db)
                .await
                .map_err(|err| {
                    error!(error=?err, user_id=?user_id, "Failed to query user");
                    HoofprintError::NeedToLogin
                })?;
            match user {
                None => {
                    error!(user_id=?user_id, "User not found");
                    if let Err(err) = session.flush().await {
                        error!(error=?err, "Failed to delete invalid session");
                    };
                    Err(HoofprintError::NeedToLogin)
                }
                Some(validuser) => Ok(crate::web::auth::AuthenticatedUser::from(validuser)),
            }
        } else {
            Err(HoofprintError::NeedToLogin)
        }
    }
}
