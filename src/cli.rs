use crate::{db::entities::user::{reset_admin_password, reset_password_by_email}, prelude::*};

use std::{num::NonZeroU16, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use sea_orm::DatabaseConnection;

#[derive(Parser, Debug)]
pub struct CliOpts {
    #[clap(long)]
    pub debug: bool,

    #[clap(long, env = "HOOFPRINT_DB_FILE", default_value = "./hoofprint.sqlite")]
    pub database_file: String,

    #[clap(long, env = "HOOFPRINT_HOST", default_value = "127.0.0.1")]
    pub host: String,

    #[clap(long, env = "HOOFPRINT_PORT", default_value_t = NonZeroU16::try_from(3000u16).expect("3000 is non-zero"))]
    pub port: NonZeroU16,

    #[clap(long, env = "HOOFPRINT_FRONTEND_HOSTNAME", default_value = "localhost")]
    pub frontend_hostname: String,

    #[clap(subcommand)]
    pub command: Option<Command>,

    #[clap(env = "HOOFPRINT_TLS_CERTIFICATE")]
    pub tls_certificate: Option<PathBuf>,

    #[clap(env = "HOOFPRINT_TLS_KEY")]
    pub tls_key: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Reset the admin user's password to a random value
    ResetAdminPassword,
    /// Reset a specific user's password by their email/username
    ResetPassword {
        /// The email address of the user
        username: String,
    },
}

pub async fn handle_admin_reset(db: DatabaseConnection) -> Result<ExitCode, ExitCode> {
    let new_password = reset_admin_password(db).await.map_err(|err| {
        error!("Failed to reset admin user: {}", err);
        ExitCode::FAILURE
    })?;

    eprintln!("Admin user has been reset.");
    eprintln!("New password: {}", new_password);

    Ok(ExitCode::SUCCESS)
}

pub async fn handle_user_reset(db: DatabaseConnection, username: String) -> Result<ExitCode, ExitCode> {
    let new_password = reset_password_by_email(&db, &username).await.map_err(|err| {
        error!("Failed to reset password for user {}: {}", username, err);
        ExitCode::FAILURE
    })?;

    eprintln!("User {} has been reset.", username);
    eprintln!("New password: {}", new_password);

    Ok(ExitCode::SUCCESS)
}
