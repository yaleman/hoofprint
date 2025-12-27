use std::path::PathBuf;

use crate::{cli::CliOpts, prelude::*};

/// A sendable configuration, for use across threads
pub type SendableConfig = Arc<RwLock<Configuration>>;

#[derive(Debug)]
pub struct Configuration {
    pub database_file: String,
    pub host: String,
    pub port: u16,
    /// Used for cookie domain and frontend URL generation
    pub frontend_hostname: String,

    pub tls_certificate: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
}

impl Configuration {
    #[cfg(test)]
    pub(crate) fn test() -> Self {
        Self {
            database_file: ":memory:".to_string(),
            host: "localhost".to_string(),
            port: 0,
            frontend_hostname: "localhost".to_string(),
            tls_certificate: None,
            tls_key: None,
        }
    }
}

impl From<&CliOpts> for Configuration {
    fn from(opts: &CliOpts) -> Self {
        Self {
            database_file: opts.database_file.clone(),
            host: opts.host.clone(),
            port: opts.port.get(),
            frontend_hostname: opts.frontend_hostname.clone(),
            tls_certificate: opts.tls_certificate.clone(),
            tls_key: opts.tls_key.clone(),
        }
    }
}

#[test]
fn test_configuration_from_cli_opts() {
    let cli_opts = CliOpts {
        database_file: "test.db".to_string(),
        host: "localhost".to_string(),
        debug: true,
        port: 12345.try_into().expect("Failed to create port"),
        frontend_hostname: "https://localhost:12345".to_string(),
        reset_admin_password: false,
        tls_certificate: None,
        tls_key: None,
    };
    let config = Configuration::from(&cli_opts);
    assert_eq!(config.database_file, "test.db");
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 12345);
    assert_eq!(config.frontend_hostname, "https://localhost:12345");
    assert!(config.tls_certificate.is_none());

    assert!(config.tls_key.is_none());
}
