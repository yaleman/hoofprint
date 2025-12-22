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

use hoofprint::prelude::*;

use std::{process::ExitCode, sync::Arc};

use clap::Parser;
use hoofprint::logging::init_logging;

#[tokio::main]
async fn main() -> Result<ExitCode, ExitCode> {
    let cli_opts = hoofprint::cli::CliOpts::parse();

    if let Some(exit_code) = init_logging(cli_opts.debug) {
        return Err(exit_code);
    }

    info!("Starting HoofPrint application");

    let config = Arc::new(RwLock::new(Configuration::from(cli_opts)));

    let db = connect(config.clone()).await.map_err(|err| {
        error!("Failed to connect to database: {}", err);
        ExitCode::FAILURE
    })?;

    info!("Connected to database successfully");

    let app_state = hoofprint::web::AppState::new(db, config);

    match hoofprint::web::start_server(app_state).await {
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
