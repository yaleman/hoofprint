//! Logging-related shenanigans
//!

use log::LevelFilter;
use std::process::ExitCode;

pub fn init_logging(debug: bool) -> Option<ExitCode> {
    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let mut logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            if debug {
                out.finish(format_args!(
                    "{} {} {} {}",
                    humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            } else {
                out.finish(format_args!(
                    "{} {} {}",
                    humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                    record.level(),
                    message
                ))
            }
        })
        .level(level)
        .chain(std::io::stdout());

    // Filter out sqlx::query events unless in debug mode
    if !debug {
        logger = logger.level_for("sqlx::query", LevelFilter::Warn);
        logger = logger.level_for("tracing::span", LevelFilter::Warn);
    }

    logger = logger.level_for("h2::codec", LevelFilter::Warn);
    if let Err(err) = logger.apply() {
        eprintln!("Failed to initialize logging: {}", err);
        Some(ExitCode::FAILURE)
    } else {
        None
    }
}
