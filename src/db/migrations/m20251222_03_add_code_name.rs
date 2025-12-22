use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251222_03_add_code_name"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Code::Table)
                    .add_column(ColumnDef::new(Code::Name).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Code::Table)
                    .drop_column(Code::Name)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum Code {
    Table,
    Name,
}
