use time::Duration;
use tokio::task::JoinHandle;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

use crate::{error::HoofprintError, web::AppState};

pub(crate) async fn create_session_layer(
    app_state: &AppState,
) -> Result<
    (
        SessionManagerLayer<SqliteStore>,
        JoinHandle<Result<(), tower_sessions::session_store::Error>>,
    ),
    HoofprintError,
> {
    let session_pool = app_state.db.clone().get_sqlite_connection_pool().clone();
    let session_store = SqliteStore::new(session_pool);
    session_store.migrate().await?;
    let cleanup_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(300)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(300)));
    Ok((session_layer, cleanup_task))
}
