//! User entity for hoofprint

use crate::{error::HoofprintError, get_random_password, password::hash_password};
use sea_orm::{ActiveValue, Condition, QueryOrder};
use sea_orm::{IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, default_value = "Uuid::new_v7()")]
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub groups: Json,
    #[serde(skip_serializing)]
    pub password: String,
}

impl Model {
    pub(crate) async fn create_new(
        db: DatabaseConnection,
        email: &str,
        display_name: &str,
        password: Option<&str>,
    ) -> Result<Model, HoofprintError> {
        let mut user = ActiveModel {
            id: ActiveValue::Set(Uuid::now_v7()),
            email: ActiveValue::Set(email.to_string()),
            display_name: ActiveValue::Set(display_name.to_string()),
            groups: ActiveValue::Set(serde_json::json!([])),
            password: ActiveValue::NotSet,
        };
        if let Some(password) = password {
            user.password = ActiveValue::Set(hash_password(password)?);
        };

        let user = user.insert(&db).await?;
        Ok(user)
    }
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

pub(crate) async fn search_users(
    db: &DatabaseConnection,
    query: &str,
) -> Result<Vec<Model>, HoofprintError> {
    let pattern = format!("%{}%", query.replace(' ', "%"));
    Entity::find()
        .filter(
            Condition::any()
                .add(Column::Email.like(&pattern))
                .add(Column::DisplayName.like(&pattern)),
        )
        .all(db)
        .await
        .map_err(|e| HoofprintError::InternalError(e.to_string()))
}

pub(crate) async fn list_users(
    db: &DatabaseConnection,
) -> Result<Vec<Model>, HoofprintError> {
    Entity::find()
        .order_by_asc(Column::Email)
        .all(db)
        .await
        .map_err(|e| HoofprintError::InternalError(e.to_string()))
}

/// Reset the admin user's password and return the new password
pub(crate) async fn reset_admin_password(db: DatabaseConnection) -> Result<String, HoofprintError> {
    let admin_id = Uuid::nil();
    reset_password_by_id(&db, admin_id).await
}

/// Reset a user's password by their email and return the new password
pub(crate) async fn reset_password_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> Result<String, HoofprintError> {
    let user = Entity::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?
        .ok_or_else(|| {
            HoofprintError::InternalError(format!("User with email {} not found", email))
        })?;

    reset_password_by_id(db, user.id).await
}

/// Helper to reset password for a given UUID
pub(crate) async fn reset_password_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<String, HoofprintError> {
    let new_password = get_random_password(16);

    let mut user = Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| HoofprintError::InternalError("User not found in DB".to_string()))?
        .into_active_model();

    user.password = ActiveValue::Set(hash_password(&new_password)?);
    user.save(db).await?;

    Ok(new_password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Configuration, db::connect, prelude::*};

    async fn setup_db() -> DatabaseConnection {
        let config = Configuration::test();
        connect(Arc::new(RwLock::new(config)))
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_list_users() {
        let db = setup_db().await;
        Model::create_new(db.clone(), "bob@example.com", "Bob Jones", None)
            .await
            .expect("Failed to create user");
        Model::create_new(db.clone(), "alice@example.com", "Alice Smith", None)
            .await
            .expect("Failed to create user");
        Model::create_new(db.clone(), "charlie@example.com", "Charlie Brown", None)
            .await
            .expect("Failed to create user");

        let users = list_users(&db).await.expect("List users failed");
        assert_eq!(users.len(), 4);
        assert_eq!(users[0].email, "admin");
        assert_eq!(users[1].email, "alice@example.com");
        assert_eq!(users[2].email, "bob@example.com");
        assert_eq!(users[3].email, "charlie@example.com");
    }

    #[tokio::test]
    async fn test_reset_password_by_email() {
        let db = setup_db().await;
        let email = "user@example.com";

        // Create a user
        Model::create_new(db.clone(), email, "User Name", Some("old-password"))
            .await
            .expect("Failed to create user");

        // Reset password
        let new_password = reset_password_by_email(&db, email)
            .await
            .expect("Password reset failed");
        assert_eq!(new_password.len(), 16);

        // Verify the user now has the new hashed password
        let user = Entity::find()
            .filter(Column::Email.eq(email))
            .one(&db)
            .await
            .expect("Failed to search for user")
            .expect("User should exist");

        assert_ne!(
            user.password,
            hash_password("old-password").expect("Failed to hash password")
        );
        assert_eq!(
            user.password,
            hash_password(&new_password).expect("Failed to hash password")
        );
    }

    #[tokio::test]
    async fn test_reset_password_by_email_not_found() {
        let db = setup_db().await;
        let result = reset_password_by_email(&db, "nonexistent@example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_password_by_id() {
        let db = setup_db().await;
        let user = Model::create_new(
            db.clone(),
            "user@example.com",
            "User Name",
            Some("old-password"),
        )
        .await
        .expect("Failed to save user");

        let new_password = reset_password_by_id(&db, user.id)
            .await
            .expect("Password reset failed");
        assert_eq!(new_password.len(), 16);

        let updated_user = Entity::find_by_id(user.id)
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User should exist");

        assert_eq!(
            updated_user.password,
            hash_password(&new_password).expect("Failed to hash password")
        );
    }

    #[tokio::test]
    async fn test_search_users() {
        let db = setup_db().await;
        Model::create_new(db.clone(), "alice@example.com", "Alice Smith", None)
            .await
            .expect("Failed to create user");
        Model::create_new(db.clone(), "bob@example.com", "Bob Jones", None)
            .await
            .expect("Failed to create user");
        Model::create_new(db.clone(), "charlie@example.com", "Charlie Brown", None)
            .await
            .expect("Failed to create user");

        // Search by email fragment
        let results = search_users(&db, "alice").await.expect("Search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "alice@example.com");

        // Search by display name fragment
        let results = search_users(&db, "Brown").await.expect("Search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, "Charlie Brown");

        // Search for something that matches multiple
        let results = search_users(&db, "e").await.expect("Search failed"); // all have @example.com
        assert_eq!(results.len(), 3);

        // Search for nothing
        let results = search_users(&db, "xyz123").await.expect("Search failed");
        assert_eq!(results.len(), 0);
    }
}
