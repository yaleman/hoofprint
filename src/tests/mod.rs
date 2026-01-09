use crate::prelude::*;
use axum_test::TestServer;
use sea_orm::DatabaseConnection;

use tokio::sync::RwLock;

use crate::{
    config::Configuration,
    db::{connect, entities::user},
    web::{AppState, server_inner},
};

pub mod codes;

pub(crate) const TEST_USER_NAME: &str = "Test User";
pub(crate) const TEST_USER_EMAIL: &str = "test@example.com";
pub(crate) const TEST_USER_PASSWORD: &str = "password";

pub(crate) async fn setup_test_user(db: DatabaseConnection) -> user::Model {
    user::Model::create_new(
        db,
        TEST_USER_EMAIL,
        TEST_USER_NAME,
        Some(TEST_USER_PASSWORD),
    )
    .await
    .expect("Failed to create test user")
}

pub(crate) async fn setup_test_server() -> (TestServer, DatabaseConnection) {
    let config = Arc::new(RwLock::new(Configuration::test()));
    let db = connect(config)
        .await
        .expect("Failed to connect to test database");

    let apptest = AppState::test_with_db(db.clone()).await;
    let _user = setup_test_user(db.clone()).await;

    let (app_server, _cleanup_task) = server_inner(apptest.clone())
        .await
        .expect("Failed to create test server");

    let mut server = TestServer::new(app_server).expect("Failed to start test server");
    server.save_cookies();

    (server, db)
}
