pub mod routes;
pub mod state;

use std::net::SocketAddr;

use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

pub use state::AppState;

/// Starts the web server with the given application state
#[instrument(level = "info", skip_all)]
pub async fn start_server(app_state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let config = app_state.config.read().await;
    let host = config.server_host.clone();
    let port = config.server_port;
    drop(config); // Release the lock

    let app = routes::routes().with_state(app_state);

    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
    info!("Starting server on http://{}", addr);

    axum_server::bind(addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await?;

    Ok(())
}
