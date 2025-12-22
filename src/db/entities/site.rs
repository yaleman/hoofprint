//! Site entity for hoofprint

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "site")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub created_at: TimeDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::code::Entity")]
    Code,
}

impl Related<super::code::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Code.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
