pub(crate) mod m20251220_01;
pub(crate) mod m20251222_01_default_admin;
pub(crate) mod m20251222_02_default_site;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(super::migrations::m20251220_01::Migration),
            Box::new(super::migrations::m20251222_01_default_admin::Migration),
            Box::new(super::migrations::m20251222_02_default_site::Migration),
        ]
    }
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::MigratorTrait;

    #[tokio::test]
    async fn test_migrator() {
        let db = crate::db::test_connect()
            .await
            .expect("Failed to connect to test DB");

        super::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
    }
}
