use crate::{cli::CliOpts, prelude::*};

/// A sendable configuration, for use across threads
pub type SendableConfig = Arc<RwLock<Configuration>>;

#[derive(Debug)]
pub struct Configuration {
    pub database_file: String,
    pub server_host: String,
    pub server_port: u16,
    /// Used for cookie domain and frontend URL generation
    pub frontend_hostname: String,
}

// impl Configuration {
//     pub fn base_url(&self) -> Result<Url, HoofprintError> {
//         use std::str::FromStr;
//         Url::from_str(&format!(
//             "http://{}:{}",
//             self.frontend_hostname, self.server_port
//         ))
//         .map_err(|err| HoofprintError::InvalidBaseUrl(err.to_string()))
//     }
// }

impl From<&CliOpts> for Configuration {
    fn from(opts: &CliOpts) -> Self {
        Self {
            database_file: opts.database_file.clone(),
            server_host: opts.host.clone(),
            server_port: opts.port.get(),
            frontend_hostname: opts.frontend_hostname.clone(),
        }
    }
}
