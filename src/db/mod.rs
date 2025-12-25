pub(crate) mod entities;
pub(crate) mod migrations;

use std::str::FromStr;

use crate::{get_random_password, prelude::*};
use migrations::Migrator;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectOptions, Database, DatabaseConnection, EntityTrait,
    JsonValue, TransactionTrait,
};
use sea_orm_migration::prelude::*;
use tracing::{info, instrument};

#[cfg(test)]
pub async fn test_connect() -> Result<DatabaseConnection, HoofprintError> {
    use std::{num::NonZeroU16, sync::Arc};

    use crate::config::Configuration;
    let config = Arc::new(RwLock::new(Configuration {
        database_file: ":memory:".to_string(),
        host: "127.0.0.1".to_string(),
        port: NonZeroU16::new(3000).expect("Invalid port number"),
        frontend_hostname: "localhost".to_string(),
        tls_certificate: None,
        tls_key: None,
    }));
    connect(config).await
}

#[instrument(level = "debug", skip_all)]
pub async fn connect(config: SendableConfig) -> Result<DatabaseConnection, HoofprintError> {
    let mut connect_options = ConnectOptions::new(get_connect_string(config).await);
    connect_options
        .sqlx_slow_statements_logging_settings(
            tracing::log::LevelFilter::Warn,
            std::time::Duration::from_secs(2),
        )
        .acquire_timeout(std::time::Duration::from_secs(10));

    let db = Database::connect(connect_options).await?;
    // start a transaction so if it doesn't work, we can roll back.
    let db_transaction = db.begin().await?;
    Migrator::up(&db_transaction, None).await?;

    // ensure the admin account exists

    if entities::user::Entity::find_by_id(Uuid::nil())
        .one(&db_transaction)
        .await?
        .is_none()
    {
        info!("Creating default admin user");
        let password = get_random_password(16);
        let admin_user = entities::user::ActiveModel {
            id: Set(Uuid::nil()),
            email: Set("admin".to_string()),
            display_name: Set("Administrator".to_string()),
            password: Set(password.clone()),
            groups: Set(JsonValue::from_str(r#"["admin"]"#)?),
        };
        admin_user.insert(&db_transaction).await?;
        info!("Default admin user created with password: {}", password);
    }

    db_transaction.commit().await?;
    Ok(db)
}

async fn get_connect_string(config: SendableConfig) -> String {
    let database_file = config.read().await.database_file.clone();

    if database_file == ":memory:" {
        info!("Using in-memory database!");
        "sqlite::memory:".to_string()
    } else {
        format!("sqlite://{database_file}?mode=rwc")
    }
}

#[tokio::test]
async fn test_test_connect() {
    test_connect()
        .await
        .expect("failed to connect to test database");
}
