use crate::{get_random_password, password::hash_password, prelude::*};

use std::{num::NonZeroU16, path::PathBuf, process::ExitCode};

use clap::Parser;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};

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

    #[clap(
        long,
        help = "Reset the admin user's password to a random value",
        action
    )]
    pub reset_admin_password: bool,

    #[clap(env = "HOOFPRINT_TLS_CERTIFICATE")]
    pub tls_certificate: Option<PathBuf>,

    #[clap(env = "HOOFPRINT_TLS_KEY")]
    pub tls_key: Option<PathBuf>,
}

pub async fn handle_admin_reset(db: DatabaseConnection) -> Result<ExitCode, ExitCode> {
    use crate::db::entities::user;

    let new_password = get_random_password(16);

    info!("Resetting admin user credentials");
    use sea_orm::EntityTrait;
    let mut admin = user::Entity::find_by_id(Uuid::nil().hyphenated())
        .one(&db)
        .await
        .map_err(|err| {
            error!("Failed to query admin user: {}", err);
            ExitCode::FAILURE
        })?
        .ok_or_else(|| {
            error!("Admin user not found");
            ExitCode::FAILURE
        })?
        .into_active_model();

    admin
        .password
        .set_if_not_equals(hash_password(&new_password).map_err(|err| {
            error!("Failed to hash password: {}", err);
            ExitCode::FAILURE
        })?);
    admin.save(&db).await.map_err(|err| {
        error!("Failed to update admin user: {}", err);
        ExitCode::FAILURE
    })?;

    println!("Admin user has been reset.");
    println!("New password: {}", new_password);

    Ok(ExitCode::SUCCESS)
}
