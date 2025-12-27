pub(crate) mod m20251220_01;
// pub(crate) mod m20251222_01_default_admin;
pub(crate) mod m20251222_02_default_site;
pub(crate) mod m20251222_03_add_code_name;
pub(crate) mod m20251224_01_username_to_email;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(super::migrations::m20251220_01::Migration),
            // Box::new(super::migrations::m20251222_01_default_admin::Migration),
            Box::new(super::migrations::m20251222_02_default_site::Migration),
            Box::new(super::migrations::m20251222_03_add_code_name::Migration),
            Box::new(super::migrations::m20251224_01_username_to_email::Migration),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sea_orm_migration::MigratorTrait;
    use tokio::sync::RwLock;

    use crate::config::Configuration;

    #[tokio::test]
    async fn test_migrator() {
        let config = Configuration::test();
        let db = crate::db::connect(Arc::new(RwLock::new(config)))
            .await
            .expect("Failed to connect to test DB");

        super::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
    }
}
