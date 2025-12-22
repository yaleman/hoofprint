use hoofprint::prelude::*;

use std::{process::ExitCode, sync::Arc};

use clap::Parser;
use hoofprint::logging::init_logging;
use tokio::sync::RwLock;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    let cli_opts = hoofprint::cli::CliOpts::parse();

    if let Some(exit_code) = init_logging(cli_opts.debug) {
        return exit_code;
    }

    info!("Starting HoofPrint application");

    let config = Arc::new(RwLock::new(hoofprint::config::Configuration {
        database_file: cli_opts.database_file,
        server_host: cli_opts.host,
        server_port: cli_opts.port,
    }));

    let db = connect(config.clone())
        .await
        .expect("failed to connect to database");

    info!("Connected to database successfully");

    let app_state = hoofprint::web::AppState::new(db, config);

    match hoofprint::web::start_server(app_state).await {
        Ok(_) => {
            info!("Server shut down gracefully");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Server error: {}", e);
            ExitCode::FAILURE
        }
    }
}
