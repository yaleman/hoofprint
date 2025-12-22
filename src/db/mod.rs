pub(crate) mod entities;
pub(crate) mod migrations;

use crate::prelude::*;
use migrations::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, TransactionTrait};
use sea_orm_migration::prelude::*;
use tracing::{info, instrument};

#[cfg(test)]
pub async fn test_connect() -> Result<DatabaseConnection, sea_orm::error::DbErr> {
    use std::sync::Arc;
    let config = Arc::new(RwLock::new(Configuration {
        database_file: ":memory:".to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: 3000,
    }));
    connect(config).await
}

#[instrument(level = "info", skip_all)]
pub async fn connect(config: SendableConfig) -> Result<DatabaseConnection, sea_orm::error::DbErr> {
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
    db_transaction.commit().await?;
    Ok(db)
}

pub async fn get_connect_string(config: SendableConfig) -> String {
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
