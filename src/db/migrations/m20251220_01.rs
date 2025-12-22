use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251220_01" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Site::Table)
                    .col(ColumnDef::new(Site::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Site::Name).string().not_null())
                    .col(ColumnDef::new(Site::Url).string().not_null())
                    .col(ColumnDef::new(Site::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(User::PreferredUsername).text().not_null())
                    .col(ColumnDef::new(User::DisplayName).text().not_null())
                    .col(ColumnDef::new(User::Groups).json().not_null())
                    .col(ColumnDef::new(User::ClaimJson).json().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Code::Table)
                    .col(ColumnDef::new(Code::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Code::UserId).uuid().not_null())
                    .col(ColumnDef::new(Code::Type).string().not_null())
                    .col(ColumnDef::new(Code::Value).string().not_null())
                    .col(ColumnDef::new(Code::SiteId).uuid().not_null())
                    .col(ColumnDef::new(Code::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Code::LastUpdated).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Code::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Code {
    Table,
    Id,
    UserId,
    Type,
    Value,
    SiteId,
    CreatedAt,
    LastUpdated,
}

#[derive(Iden)]
pub enum Site {
    Table,
    Id,
    Name,
    Url,
    CreatedAt,
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    PreferredUsername,
    DisplayName,
    Groups,
    ClaimJson,
}
