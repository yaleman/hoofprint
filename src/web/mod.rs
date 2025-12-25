pub(crate) mod auth;
pub(crate) mod forms;
pub(crate) mod manifest;
pub mod routes;
pub mod sessions;
pub mod state;
pub(crate) mod views;

use std::{net::SocketAddr, path::PathBuf};

use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};
use tracing::{error, info, instrument};

pub use state::AppState;

use crate::error::HoofprintError;

/// Starts the web server with the given application state
#[instrument(level = "debug", skip_all)]
pub async fn start_server(app_state: AppState) -> Result<(), HoofprintError> {
    let config = app_state.config.read().await;
    let host = config.host.clone();
    let port = config.port;
    let frontend_hostname = config.frontend_hostname.clone();
    let tls_certificate = config.tls_certificate.clone();
    let tls_key = config.tls_key.clone();
    drop(config); // Release the lock

    let (session_layer, cleanup_task) = sessions::create_session_layer(&app_state).await?;

    let compression_layer = CompressionLayer::new()
        .gzip(true)
        .deflate(true)
        .quality(tower_http::CompressionLevel::Best);

    let app = routes::routes()
        .with_state(app_state)
        .layer(session_layer)
        .nest_service(
            "/static",
            ServeDir::new(PathBuf::from("./static/")).precompressed_br(),
        )
        .layer(compression_layer);

    let addr = format!("{}:{}", host, port);
    let addr = addr.parse::<SocketAddr>().map_err(|err| {
        error!(address=?addr, error=?err, "Failed to parse listener address");
        err
    })?;

    match (tls_certificate, tls_key) {
        (Some(cert_path), Some(key_path)) => {
            info!("Starting server on https://{}:{}", frontend_hostname, port);
            rustls::crypto::aws_lc_rs::default_provider().install_default()?;

            use axum_server::tls_rustls::RustlsConfig;

            let tls_config = RustlsConfig::from_pem_file(&cert_path, &key_path)
                .await
                .map_err(|e| {
                    error!("Failed to load TLS certificates: {:?}", e);
                    error!("  Certificate: {}", cert_path.display());
                    error!("  Key: {}", key_path.display());
                    HoofprintError::from(e)
                })?;

            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service())
                .await
                .map_err(|e| {
                    error!("Server error: {:?}", e);
                    HoofprintError::from(e)
                })?;
        }
        _ => {
            info!("Starting server on http://{}:{}", frontend_hostname, port);

            axum_server::bind(addr)
                .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
                .await?;
        }
    }

    cleanup_task.await??;
    Ok(())
}
