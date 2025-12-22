use crate::prelude::*;
use sea_orm::DatabaseConnection;

/// Application state shared across all web handlers
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: SendableConfig,
}

impl AppState {
    pub fn new(db: DatabaseConnection, config: SendableConfig) -> Self {
        Self { db, config }
    }
}
