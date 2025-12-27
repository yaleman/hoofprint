use crate::prelude::*;
use axum::http::{HeaderMap, StatusCode, header::CONTENT_TYPE};

/// The MIME type for `.webmanifest` files.
const MIME_TYPE_MANIFEST: &str = "application/manifest+json;charset=utf-8";

#[allow(dead_code)] // because not all variants may be used immediately
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DisplayOption {
    Standalone,
    #[serde(rename = "minimal-ui")]
    MinimalUi,
    Fullscreen,
    Browser,
}

#[allow(dead_code)] // because not all variants may be used immediately
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum IconPurpose {
    Any,
    Maskable,
    Monochrome,
}

#[derive(Serialize)]
pub(crate) struct ManifestIcon {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<IconPurpose>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub type_: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ManifestResponse {
    name: &'static str,
    icons: Vec<ManifestIcon>,
    start_url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    display: Option<DisplayOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_override: Option<Vec<String>>,
}

#[instrument(level = "debug", skip_all)]
pub(crate) async fn manifest(
    State(app_state): State<AppState>,
    _headers: HeaderMap,
) -> Result<impl IntoResponse, HoofprintError> {
    let manifest = ManifestResponse {
        name: "hoofPrint",
        start_url: app_state.base_url.parse()?,
        display: Some(DisplayOption::MinimalUi),
        display_override: None,
        icons: vec![ManifestIcon {
            src: "/static/img/128x128logo.png".to_string(),
            sizes: Some("128x128".to_string()),
            purpose: Some(IconPurpose::Any),
            type_: Some("image/png".to_string()),
        }],
    };
    let res = (
        StatusCode::OK,
        [(CONTENT_TYPE, MIME_TYPE_MANIFEST)],
        Json(manifest),
    );
    Ok(res)
}

#[tokio::test]
async fn test_manifest_response() {
    let app_state = AppState::test().await;
    let response = manifest(State(app_state), HeaderMap::new())
        .await
        .expect("Failed to get manifest response");
    let (parts, _body) = response.into_response().into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    assert_eq!(
        parts
            .headers
            .get(CONTENT_TYPE)
            .expect("Content-Type header missing")
            .to_str()
            .expect("Failed to convert Content-Type header to str"),
        MIME_TYPE_MANIFEST
    );
}
