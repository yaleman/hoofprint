#![deny(warnings)]
#![warn(unused_extern_crates)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unreachable)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::trivially_copy_pass_by_ref)]

use hoofprint::{cli::handle_admin_reset, config::Configuration, prelude::*};

use std::{process::ExitCode, sync::Arc};

use clap::Parser;
use hoofprint::logging::init_logging;

#[tokio::main]
async fn main() -> Result<ExitCode, ExitCode> {
    let cli_opts = hoofprint::cli::CliOpts::parse();

    if let Some(exit_code) = init_logging(cli_opts.debug) {
        return Err(exit_code);
    }
    let config = Arc::new(RwLock::new(Configuration::from(&cli_opts)));

    let db = connect(config.clone()).await.map_err(|err| {
        error!("Failed to connect to database: {}", err);
        ExitCode::FAILURE
    })?;

    debug!("Connected to database successfully");

    if cli_opts.reset_admin_password {
        return handle_admin_reset(db.clone()).await;
    }

    let app_state = hoofprint::web::AppState::new(db, config);

    tokio::select!(
        biased;

        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl-C, shutting down");
            Ok(ExitCode::SUCCESS)
        }
         Some(()) = async move {
            let sigterm = tokio::signal::unix::SignalKind::alarm();
            #[allow(clippy::unwrap_used)]
            tokio::signal::unix::signal(sigterm).unwrap().recv().await
        } => {
            info!("Server shut down gracefully");
            Ok(ExitCode::SUCCESS)

        }

        result = hoofprint::web::start_server(app_state) => {
            match result {
                Ok(_) => {
                    info!("Server shut down gracefully");
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    eprintln!("Server error: {}", e);
                    Err(ExitCode::FAILURE)
                }
            }
        }
    )
}
