//! Tidal proxy app-token fetch.

use serde_json::Value;

use crate::errors::PlaylistImportError;
use crate::http::http;

pub(super) async fn get_app_token() -> Result<String, PlaylistImportError> {
    // Proxy handles credentials
    let url = format!("{}/tidal/token", crate::QBZ_PROXY_BASE);

    let response: Value = http()
        .get(&url)
        .header(reqwest::header::USER_AGENT, crate::http::USER_AGENT)
        .send()
        .await
        .map_err(|e| PlaylistImportError::Http(e.to_string()))?
        .json()
        .await
        .map_err(|e| PlaylistImportError::Parse(e.to_string()))?;

    response
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .ok_or_else(|| PlaylistImportError::Parse("Tidal proxy missing access_token".to_string()))
}
