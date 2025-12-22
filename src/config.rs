use crate::{cli::CliOpts, prelude::*};

/// A sendable configuration, for use across threads
pub type SendableConfig = Arc<RwLock<Configuration>>;

pub struct Configuration {
    pub database_file: String,
    pub server_host: String,
    pub server_port: u16,
}

impl From<CliOpts> for Configuration {
    fn from(opts: CliOpts) -> Self {
        Self {
            database_file: opts.database_file,
            server_host: opts.host,
            server_port: opts.port,
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            database_file: "./hoofprint.sqlite".to_string(),
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
        }
    }
}
