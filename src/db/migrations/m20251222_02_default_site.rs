use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use sea_orm_migration::prelude::*;
use uuid::Uuid;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251222_02_default_site"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Use SeaORM entities to properly insert the default site
        // This ensures UUIDs are stored in the correct format
        let db = manager.get_connection();

        // Check if site already exists
        let existing = crate::db::entities::site::Entity::find_by_id(Uuid::nil())
            .one(db)
            .await?;

        if existing.is_none() {
            // Create the default site using ActiveModel
            let default_site = crate::db::entities::site::ActiveModel {
                id: Set(Uuid::nil()),
                name: Set("Generic Site".to_string()),
                url: Set(String::new()),
                created_at: Set(time::PrimitiveDateTime::new(
                    time::OffsetDateTime::now_utc().date(),
                    time::OffsetDateTime::now_utc().time(),
                )),
            };

            default_site.insert(db).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Delete the default site using SeaORM
        let db = manager.get_connection();

        crate::db::entities::site::Entity::delete_by_id(Uuid::nil())
            .exec(db)
            .await?;

        Ok(())
    }
}
