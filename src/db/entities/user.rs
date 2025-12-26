//! User entity for hoofprint

use sea_orm::{IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::HoofprintError, get_random_password, password::hash_password};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub groups: Json,
    #[serde(skip_serializing)]
    pub password: String,
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

/// Reset the admin user's password and return the new password
pub(crate) async fn reset_admin_password(db: DatabaseConnection) -> Result<String, HoofprintError> {
    let new_password = get_random_password(16);

    info!("Resetting admin user credentials");
    let mut admin = Entity::find_by_id(Uuid::nil().hyphenated())
        .one(&db)
        .await?
        .ok_or_else(|| HoofprintError::InternalError("Admin user not found in DB".to_string()))?
        .into_active_model();

    admin
        .password
        .set_if_not_equals(hash_password(&new_password)?);
    admin.save(&db).await?;
    Ok(new_password)
}
