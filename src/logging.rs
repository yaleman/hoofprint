//! Logging-related shenanigans
//!

use std::process::ExitCode;

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

pub fn init_logging(debug: bool) -> Option<ExitCode> {
    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    if let Err(err) = CombinedLogger::init(vec![TermLogger::new(
        level,
        ConfigBuilder::new().set_target_level(level).build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]) {
        eprintln!("Failed to initialize logging: {}", err);
        Some(ExitCode::FAILURE)
    } else {
        None
    }
}
