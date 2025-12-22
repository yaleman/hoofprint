use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    Code,
    db::entities::{code, site},
    error::HoofprintError,
    web::{
        auth::AuthenticatedUser,
        forms::{CreateCodeForm, EditCodeForm},
        state::AppState,
    },
};

#[derive(Template)]
#[template(path = "index.html")]
struct HomePage {
    codes: Vec<CodeListItem>,
}

struct CodeListItem {
    id: Uuid,
    code_type: String,
    code_value: String,
    code_name: Option<String>,
    site_name: String,
    created_at: String,
}

/// Homepage handler that returns a simple HTML response
#[instrument(level = "info")]
pub(crate) async fn homepage(
    State(app_state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Html<String>, HoofprintError> {
    // Query all codes for the authenticated user with related sites
    let codes_with_sites = code::Entity::find()
        .filter(code::Column::UserId.eq(auth.user_id))
        .find_also_related(site::Entity)
        .all(&app_state.db)
        .await?;

    // Transform into template-friendly structure
    let codes = codes_with_sites
        .into_iter()
        .map(|(code_model, site_model)| {
            let site_name = site_model
                .map(|s| s.name)
                .unwrap_or_else(|| "Unknown Site".to_string());

            CodeListItem {
                id: code_model.id,
                code_type: code_model.type_,
                code_value: code_model.value,
                code_name: code_model.name.clone(),
                site_name,
                created_at: code_model.created_at.to_string(),
            }
        })
        .collect();

    let homepage = HomePage { codes };
    Ok(Html(homepage.render()?))
}

#[derive(Template)]
#[template(path = "view_code.html")]
#[allow(dead_code)]
struct ViewCodePage {
    pub code: Code,
    pub code_id: Uuid,
    pub code_value: String,
    pub code_name: Option<String>,
    pub site_name: String,
    pub created_at: String,
    pub last_updated: Option<String>,
    pub is_owner: bool,
}

#[instrument(level = "info")]
pub(crate) async fn view_code(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    auth: AuthenticatedUser,
) -> Result<Html<String>, HoofprintError> {
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Fetch code from database with related site
    let code_with_site = code::Entity::find_by_id(code_id)
        .find_also_related(site::Entity)
        .one(&app_state.db)
        .await?
        .ok_or_else(|| HoofprintError::NotFound(format!("Code {}", code_id)))?;

    let (code_model, site_model) = code_with_site;
    let site_model = site_model.ok_or_else(|| HoofprintError::InvalidSite)?;

    // Convert database code to display Code enum

    let code = Code::try_from(&code_model)?;

    let code_page = ViewCodePage {
        code,
        code_id: code_model.id,
        code_value: code_model.value.clone(),
        code_name: code_model.name.clone(),
        site_name: site_model.name,
        created_at: code_model.created_at.to_string(),
        last_updated: code_model.last_updated.map(|dt| dt.to_string()),
        is_owner: code_model.user_id == auth.user_id,
    };

    Ok(Html(code_page.render()?))
}

#[derive(Template)]
#[template(path = "create_code.html")]
struct CreateCodePage {
    pub sites: Vec<SiteOption>,
    pub error: Option<String>,
    pub uuid_nil: String,
}

struct SiteOption {
    pub id: String,
    pub name: String,
}

#[instrument(level = "info")]
pub(crate) async fn create_code_get(
    State(app_state): State<AppState>,
    _auth: AuthenticatedUser,
) -> Result<Html<String>, HoofprintError> {
    // Fetch all sites for dropdown
    let sites_models = site::Entity::find().all(&app_state.db).await?;

    // Transform into template structure
    let sites = sites_models
        .into_iter()
        .map(|site| SiteOption {
            id: site.id.to_string(),
            name: site.name,
        })
        .collect();

    let page = CreateCodePage {
        sites,
        error: None,
        uuid_nil: Uuid::nil().to_string(),
    };

    Ok(Html(page.render()?))
}

#[instrument(level = "info")]
pub(crate) async fn create_code_post(
    State(app_state): State<AppState>,
    auth: AuthenticatedUser,
    Form(form): Form<CreateCodeForm>,
) -> Result<Redirect, HoofprintError> {
    // Validate form data
    form.validate()?;

    // Parse site_id
    let site_id = form.parse_site_id()?;

    // Verify site exists
    site::Entity::find_by_id(site_id)
        .one(&app_state.db)
        .await?
        .ok_or_else(|| {
            HoofprintError::ValidationError(vec![format!("Site {} not found", site_id)])
        })?;

    // Create new Code
    let new_code_id = Uuid::now_v7();

    // Convert empty string to None for name field
    let name = if form.code_name.as_ref().is_none_or(|s| s.is_empty()) {
        None
    } else {
        form.code_name
    };

    let new_code = code::ActiveModel {
        id: Set(new_code_id),
        user_id: Set(auth.user_id),
        type_: Set(form.code_type),
        value: Set(form.code_value),
        name: Set(name),
        site_id: Set(site_id),
        created_at: Set(time::PrimitiveDateTime::new(
            time::OffsetDateTime::now_utc().date(),
            time::OffsetDateTime::now_utc().time(),
        )),
        last_updated: Set(None),
    };

    // Insert into database
    new_code.insert(&app_state.db).await?;

    // Redirect to view page
    Ok(Redirect::to(&format!("/view/{}", new_code_id)))
}

#[derive(Template)]
#[template(path = "edit_code.html")]
struct EditCodePage {
    pub code_id: String,
    pub code_type: String,
    pub code_value: String,
    pub code_name: Option<String>,
    pub site_id: String,
    pub sites: Vec<SiteOption>,
    pub created_at: String,
    pub last_updated: Option<String>,
    pub error: Option<String>,
    #[allow(dead_code)]
    pub uuid_nil: Uuid,
}

#[instrument(level = "info")]
pub(crate) async fn edit_code_get(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    auth: AuthenticatedUser,
) -> Result<Html<String>, HoofprintError> {
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Fetch code from database
    let code_model = code::Entity::find_by_id(code_id)
        .one(&app_state.db)
        .await?
        .ok_or_else(|| HoofprintError::NotFound(format!("Code {}", code_id)))?;

    // Verify ownership
    if code_model.user_id != auth.user_id {
        return Err(HoofprintError::Unauthorized);
    }

    // Fetch all sites for dropdown
    let sites_models = site::Entity::find()
        // .filter(site::Column::Id.ne(Uuid::nil().to_string()))
        .all(&app_state.db)
        .await?;

    // Transform sites
    let sites = sites_models
        .into_iter()
        .map(|site| SiteOption {
            id: site.id.to_string(),
            name: site.name,
        })
        .collect();

    // Create page data with pre-filled values
    let page = EditCodePage {
        code_id: code_model.id.to_string(),
        code_type: code_model.type_,
        code_value: code_model.value,
        code_name: code_model.name.clone(),
        site_id: code_model.site_id.to_string(),
        sites,
        uuid_nil: Uuid::nil(),
        created_at: code_model.created_at.to_string(),
        last_updated: code_model.last_updated.map(|dt| dt.to_string()),
        error: None,
    };

    Ok(Html(page.render()?))
}

#[instrument(level = "info")]
pub(crate) async fn edit_code_post(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    auth: AuthenticatedUser,
    Form(form): Form<EditCodeForm>,
) -> Result<Redirect, HoofprintError> {
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Validate form data
    form.validate()?;

    // Parse site_id
    let site_id = form.parse_site_id()?;

    // Fetch existing code from database
    let code_model = code::Entity::find_by_id(code_id)
        .one(&app_state.db)
        .await?
        .ok_or_else(|| HoofprintError::NotFound(format!("Code {}", code_id)))?;

    // Verify user owns this code
    if code_model.user_id != auth.user_id {
        return Err(HoofprintError::Unauthorized);
    }

    // Verify site exists
    site::Entity::find_by_id(site_id)
        .one(&app_state.db)
        .await?
        .ok_or_else(|| {
            HoofprintError::ValidationError(vec![format!("Site {} not found", site_id)])
        })?;

    // Update code
    let mut code_active: code::ActiveModel = code_model.into();

    // Convert empty string to None for name field
    let name = if form.code_name.as_ref().is_none_or(|s| s.is_empty()) {
        None
    } else {
        form.code_name
    };

    code_active.type_ = Set(form.code_type);
    code_active.value = Set(form.code_value);
    code_active.name = Set(name);
    code_active.site_id = Set(site_id);
    code_active.last_updated = Set(Some(time::PrimitiveDateTime::new(
        time::OffsetDateTime::now_utc().date(),
        time::OffsetDateTime::now_utc().time(),
    )));

    code_active.update(&app_state.db).await?;

    // Redirect to view page
    Ok(Redirect::to(&format!("/view/{}", code_id)))
}

#[instrument(level = "info")]
pub(crate) async fn code_delete(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    auth: AuthenticatedUser,
) -> Result<Redirect, HoofprintError> {
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Fetch code from database
    let code_model = code::Entity::find_by_id(code_id)
        .one(&app_state.db)
        .await?
        .ok_or_else(|| HoofprintError::NotFound(format!("Code {}", code_id)))?;

    // Verify user owns this code
    if code_model.user_id != auth.user_id {
        return Err(HoofprintError::Unauthorized);
    }

    // Delete code from database
    code::Entity::delete_by_id(code_id)
        .exec(&app_state.db)
        .await?;

    // Redirect to homepage (code no longer exists)
    Ok(Redirect::to("/"))
}
