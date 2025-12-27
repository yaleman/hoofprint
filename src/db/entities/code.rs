//! A code is a single barcode/identifier that is associated with a user/site

use sea_orm::{ActiveValue::Set, entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

use crate::{Code, error::HoofprintError};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "code")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    #[sea_orm(column_name = "type")]
    pub type_: String,
    pub value: String,
    pub site_id: Uuid,
    pub created_at: DateTimeUtc,
    pub last_updated: Option<DateTimeUtc>,
    pub name: Option<String>,
}

impl Model {
    /// Create a new code model
    pub async fn create_new(
        db: DatabaseConnection,
        user_id: Uuid,
        type_: Code,
        value: &str,
        site_id: Uuid,
        name: Option<&str>,
    ) -> Result<Model, HoofprintError> {
        ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(user_id),
            type_: Set(type_.to_string()),
            value: Set(value.to_string()),
            site_id: Set(site_id),
            created_at: Set(chrono::Utc::now()),
            last_updated: Set(None),
            name: Set(name.map(|n| n.to_string())),
        }
        .insert(&db)
        .await
        .map_err(HoofprintError::from)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::site::Entity",
        from = "Column::SiteId",
        to = "super::site::Column::Id"
    )]
    Site,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::site::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
