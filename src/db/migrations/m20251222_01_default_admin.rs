use sea_orm_migration::prelude::*;
use uuid::Uuid;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251222_01_default_admin"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Insert default admin user with UUID 00000000-0000-0000-0000-000000000000
        // Using INSERT OR IGNORE for idempotency (SQLite syntax)
        let sql = r#"
            INSERT OR IGNORE INTO "user" (id, preferred_username, display_name, groups, claim_json)
            VALUES ('XXXXX', 'admin', 'Default Administrator', '[]', '{}')
        "#
        .replace("XXXXX", &(Uuid::nil().to_string()));

        manager.get_connection().execute_unprepared(&sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Delete the default admin user
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(User::Table)
                    .and_where(Expr::col(User::Id).eq(&(Uuid::nil().to_string())))
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
#[allow(dead_code)]
enum User {
    Table,
    Id,
    PreferredUsername,
    DisplayName,
    Groups,
    ClaimJson,
}
