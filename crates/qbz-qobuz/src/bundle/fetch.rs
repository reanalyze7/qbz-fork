use super::parse::{extract_app_id, extract_bundle_url, extract_private_key, extract_secrets};
use super::{BundleTokens, BUNDLE_BASE_URL, BUNDLE_FETCH_TIMEOUT, LOGIN_PAGE_URL};
use crate::error::{ApiError, Result};
use reqwest::Client;

/// Parse the bundle version out of the `/resources/<version>/bundle.js` path,
/// e.g. `/resources/8.1.0-b019/bundle.js` -> `8.1.0-b019`.
fn bundle_version_from_url(bundle_url: &str) -> String {
    bundle_url
        .trim_start_matches("/resources/")
        .trim_end_matches("/bundle.js")
        .to_string()
}

/// Fetch the login page and return the current bundle URL + parsed version.
/// Cheap (~small page); used both by the full extraction and the background
/// version check.
pub(super) async fn fetch_bundle_url(client: &Client) -> Result<(String, String)> {
    let login_page = client
        .get(LOGIN_PAGE_URL)
        .timeout(BUNDLE_FETCH_TIMEOUT)
        .send()
        .await?
        .text()
        .await?;
    let bundle_url = extract_bundle_url(&login_page)?;
    let version = bundle_version_from_url(&bundle_url);
    Ok((bundle_url, version))
}

/// Single network extraction attempt: fetch login page -> bundle.js -> parse.
/// Returns the tokens together with the bundle version they came from.
pub(super) async fn extract_bundle_tokens_once(client: &Client) -> Result<(BundleTokens, String)> {
    // Step 1: Get login page to find bundle URL + version
    let (bundle_url, version) = fetch_bundle_url(client).await?;
    let full_bundle_url = format!("{}{}", BUNDLE_BASE_URL, bundle_url);

    // Step 2: Fetch the bundle (large; bounded by BUNDLE_FETCH_TIMEOUT)
    let bundle_content = client
        .get(&full_bundle_url)
        .timeout(BUNDLE_FETCH_TIMEOUT)
        .send()
        .await?
        .text()
        .await?;

    // Step 3: Extract app_id
    let app_id = extract_app_id(&bundle_content)?;

    // Step 4: Extract secrets
    let secrets = extract_secrets(&bundle_content)?;

    if secrets.is_empty() {
        return Err(ApiError::BundleExtractionError(
            "No secrets found in bundle".to_string(),
        ));
    }

    // Step 5: Extract OAuth private_key (optional - present in newer bundles)
    let private_key = extract_private_key(&bundle_content);
    if private_key.is_some() {
        log::info!("OAuth private_key extracted from bundle");
    } else {
        log::debug!("OAuth private_key not found in bundle (older bundle version)");
    }

    Ok((
        BundleTokens {
            app_id,
            secrets,
            private_key,
        },
        version,
    ))
}

/// Backwards-compatible one-shot extraction (no caching). Retained for callers
/// that just want a live fetch; the app startup path uses
/// [`extract_and_cache_bundle_tokens`] instead.
pub async fn extract_bundle_tokens(client: &Client) -> Result<BundleTokens> {
    extract_bundle_tokens_once(client).await.map(|(t, _)| t)
}
