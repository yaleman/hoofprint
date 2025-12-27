use std::collections::HashMap;

use crate::{db::entities::user, prelude::*, web::auth::AUTH_USER_ID};

/// Application state shared across all web handlers
#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: SendableConfig,
    pub etags: Arc<RwLock<HashMap<String, String>>>,
    pub base_url: String,
}

impl AppState {
    pub async fn new(db: DatabaseConnection, config: SendableConfig) -> Self {
        let config_reader = config.read().await;
        let scheme = if config_reader.tls_certificate.is_some() && config_reader.tls_key.is_some() {
            "https"
        } else {
            "http"
        };
        let port = match config_reader.port {
            80 | 443 => "".to_string(),
            other => format!(":{}", other),
        };
        let base_url = format!("{}://{}{}", scheme, config_reader.frontend_hostname, port);
        drop(config_reader); // Release the lock
        Self {
            db,
            config,
            etags: Arc::new(RwLock::new(HashMap::new())),
            base_url,
        }
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

    #[cfg(test)]
    pub(crate) async fn test() -> Self {
        let config = Arc::new(RwLock::new(crate::config::Configuration::test()));
        let db = crate::db::connect(config.clone())
            .await
            .expect("Failed to connect to test database!");
        Self::new(db, config).await
    }

    #[cfg(test)]
    pub(crate) async fn test_with_db(db: DatabaseConnection) -> Self {
        let config = Arc::new(RwLock::new(crate::config::Configuration::test()));
        Self::new(db, config).await
    }
}
