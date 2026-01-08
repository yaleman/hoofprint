use std::time::SystemTime;

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    body::Bytes,
    extract::{Form, Path, State},
    response::Redirect,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, prelude::DateTimeUtc,
};
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    Code,
    db::entities::{code, site},
    error::HoofprintError,
    web::{
        forms::{CreateCodeForm, EditCodeForm},
        state::AppState,
    },
};

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub(crate) struct HomePage {
    codes: Vec<CodeListItem>,
    user_name: String,
    user_email: String,
}

struct CodeListItem {
    id: Uuid,
    // code_type: String,
    code_value: String,
    code_name: Option<String>,
    site_name: String,
}

/// Homepage handler that returns a simple HTML response
#[instrument(level = "debug", skip_all)]
pub(crate) async fn homepage(
    State(app_state): State<AppState>,
    session: Session,
) -> Result<HomePage, HoofprintError> {
    // Query all codes for the authenticated user with related sites
    let auth = app_state.get_authenticated_user(&session).await?;

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
                code_value: code_model.value,
                code_name: code_model.name.clone(),
                site_name,
            }
        })
        .collect();

    Ok(HomePage {
        codes,
        user_name: auth.display_name,
        user_email: auth.email,
    })
}

#[derive(Template, WebTemplate)]
#[template(path = "view_code.html")]
// #[allow(dead_code)]
pub(crate) struct ViewCodePage {
    pub code: Code,
    pub code_id: Uuid,
    pub code_value: String,
    pub code_name: Option<String>,
}

#[instrument(level = "debug", skip(app_state, session))]
pub(crate) async fn view_code(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    session: Session,
) -> Result<ViewCodePage, HoofprintError> {
    let auth = app_state.get_authenticated_user(&session).await?;
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Fetch code from database with related site
    let code_with_site = code::Entity::find_by_id(code_id)
        .filter(code::Column::UserId.eq(auth.user_id))
        .find_also_related(site::Entity)
        .one(&app_state.db)
        .await?
        .ok_or_else(|| HoofprintError::NotFound(format!("Code {}", code_id)))?;

    let (code_model, site_model) = code_with_site;
    let _site_model = site_model.ok_or_else(|| HoofprintError::InvalidSite)?;

    // Convert database code to display Code enum

    let code = Code::try_from(&code_model)?;

    let code_page = ViewCodePage {
        code,
        code_id: code_model.id,
        code_value: code_model.value.clone(),
        code_name: code_model.name.clone(),
        // site_name: site_model.name,
        // created_at: code_model.created_at.to_string(),
        // last_updated: code_model.last_updated.map(|dt| dt.to_string()),
        // is_owner: code_model.user_id == auth.user_id,
    };

    Ok(code_page)
}

#[derive(Template, WebTemplate)]
#[template(path = "create_code.html")]
pub(crate) struct CreateCodePage {
    pub sites: Vec<SiteOption>,
    pub error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "scan.html")]
pub(crate) struct ScanCodePage {
    pub sites: Vec<SiteOption>,
    pub error: Option<String>,
    pub uuid_nil: String,
}

pub(crate) struct SiteOption {
    pub id: String,
    pub name: String,
}

#[instrument(level = "debug")]
pub(crate) async fn create_code_get(
    State(app_state): State<AppState>,
    session: Session,
) -> Result<CreateCodePage, HoofprintError> {
    app_state.get_authenticated_user(&session).await?;

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

    Ok(CreateCodePage { sites, error: None })
}

#[instrument(level = "debug")]
pub(crate) async fn create_code_post(
    State(app_state): State<AppState>,
    session: Session,
    Form(form): Form<CreateCodeForm>,
) -> Result<Redirect, HoofprintError> {
    let auth = app_state.get_authenticated_user(&session).await?;
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
        created_at: Set(DateTimeUtc::from(SystemTime::now())),
        last_updated: Set(None),
    };

    // Insert into database
    new_code.insert(&app_state.db).await?;

    // Redirect to view page
    Ok(Redirect::to(&format!("/view/{}", new_code_id)))
}

#[derive(Template, WebTemplate)]
#[template(path = "edit_code.html")]
pub(crate) struct EditCodePage {
    pub code_id: String,
    pub code_type: String,
    pub code_value: String,
    pub code_name: Option<String>,
    pub site_id: String,
    pub sites: Vec<SiteOption>,
    pub created_at: String,
    pub last_updated: Option<String>,
    pub error: Option<String>,
}

#[instrument(level = "debug", skip(app_state, session))]
pub(crate) async fn edit_code_get(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    session: Session,
) -> Result<EditCodePage, HoofprintError> {
    let auth = app_state.get_authenticated_user(&session).await?;
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Fetch code from database
    let code_model = code::Entity::find_by_id(code_id)
        .filter(code::Column::UserId.eq(auth.user_id))
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
        created_at: code_model.created_at.to_string(),
        last_updated: code_model.last_updated.map(|dt| dt.to_string()),
        error: None,
    };

    Ok(page)
}

#[instrument(level = "debug", skip(app_state, session, form))]
pub(crate) async fn edit_code_post(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    session: Session,
    Form(form): Form<EditCodeForm>,
) -> Result<Redirect, HoofprintError> {
    let auth = app_state.get_authenticated_user(&session).await?;
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Validate form data
    form.validate()?;

    // Parse site_id
    let site_id = form.parse_site_id()?;

    // Fetch existing code from database
    let code_model = code::Entity::find_by_id(code_id)
        .filter(code::Column::UserId.eq(auth.user_id))
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
    code_active.last_updated = Set(Some(DateTimeUtc::from(SystemTime::now())));

    code_active.update(&app_state.db).await?;

    // Redirect to view page
    Ok(Redirect::to(&format!("/view/{}", code_id)))
}

#[instrument(level = "debug", skip(app_state, session))]
pub(crate) async fn code_delete(
    State(app_state): State<AppState>,
    Path(code_id_str): Path<String>,
    session: Session,
) -> Result<Redirect, HoofprintError> {
    let auth = app_state.get_authenticated_user(&session).await?;
    // Parse code_id as UUID
    let code_id = Uuid::parse_str(&code_id_str)
        .map_err(|_| HoofprintError::NotFound(format!("Invalid code ID: {}", code_id_str)))?;

    // Fetch code from database
    let code_model = code::Entity::find_by_id(code_id)
        .filter(code::Column::UserId.eq(auth.user_id))
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

#[instrument(level = "debug", skip_all)]
pub(crate) async fn scan_get(
    State(app_state): State<AppState>,
    session: Session,
) -> Result<ScanCodePage, HoofprintError> {
    app_state.get_authenticated_user(&session).await?;

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

    Ok(ScanCodePage {
        sites,
        error: None,
        uuid_nil: Uuid::nil().to_string(),
    })
}

#[instrument(level = "debug", skip_all, fields(site_id = %form.site_id, code_type = %form.code_type))]
pub(crate) async fn scan_post(
    State(app_state): State<AppState>,
    session: Session,
    Form(form): Form<CreateCodeForm>,
) -> Result<Redirect, HoofprintError> {
    let auth = app_state.get_authenticated_user(&session).await?;
    // Validate form data
    form.validate()?;

    // Parse site_id, default to Uuid::nil() if empty or already nil
    let site_id = form.parse_site_id()?;
    let site_id = if site_id == Uuid::nil() || form.site_id.is_empty() {
        Uuid::nil()
    } else {
        // Verify site exists if not using default
        site::Entity::find_by_id(site_id)
            .one(&app_state.db)
            .await?
            .ok_or_else(|| {
                HoofprintError::ValidationError(vec![format!("Site {} not found", site_id)])
            })?;
        site_id
    };

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
        created_at: Set(DateTimeUtc::from(SystemTime::now())),
        last_updated: Set(None),
    };

    // Insert into database
    new_code.insert(&app_state.db).await?;

    // Redirect to view page
    Ok(Redirect::to(&format!("/view/{}", new_code_id)))
}

pub(crate) async fn csp_report_only(body: Bytes) -> Result<(), HoofprintError> {
    // For now, just log that a report was received.
    // In a real application, you would parse and store the report details.
    let body_string = String::from_utf8_lossy(&body);
    let body_json = serde_json::from_str::<serde_json::Value>(&body_string);
    #[allow(clippy::expect_used)]
    if let Ok(json) = body_json {
        tracing::info!(
            "CSP Report-Only violation reported:\n {}",
            serde_json::to_string_pretty(&json).expect("Failed to pretty-print JSON")
        );
    } else {
        tracing::info!("CSP Report-Only violation reported: {:?}", body_string);
    }
    Ok(())
}
