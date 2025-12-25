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

    let mut config_builder = ConfigBuilder::new();
    config_builder.set_target_level(level);

    // Filter out sqlx::query events unless in debug mode
    if !debug {
        config_builder.add_filter_ignore_str("sqlx::query");
        config_builder.add_filter_ignore_str("call");
    }

    config_builder.add_filter_ignore_str("h2::codec");

    if let Err(err) = CombinedLogger::init(vec![TermLogger::new(
        level,
        config_builder.build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]) {
        eprintln!("Failed to initialize logging: {}", err);
        Some(ExitCode::FAILURE)
    } else {
        None
    }
}
