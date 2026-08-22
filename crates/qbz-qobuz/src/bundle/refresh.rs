use super::cache::{load_cached_bundle, now_unix, save_cached_bundle};
use super::fetch::{extract_bundle_tokens_once, fetch_bundle_url};
use super::{BundleTokens, CachedBundle, BUNDLE_EXTRACTION_RETRIES};
use crate::error::{ApiError, Result};
use reqwest::Client;
use std::time::Duration;

/// Extract app_id, secrets, and OAuth private_key from the live Qobuz bundle,
/// with a small retry loop, and persist the result to the on-disk cache.
///
/// This is the network ("cold") path. Prefer [`load_cached_bundle`] +
/// [`refresh_bundle_if_changed`] on warm starts so the UI never blocks on the
/// 7 MB download.
pub async fn extract_and_cache_bundle_tokens(client: &Client) -> Result<BundleTokens> {
    let mut last_err: Option<ApiError> = None;
    let attempts = BUNDLE_EXTRACTION_RETRIES + 1;
    for attempt in 1..=attempts {
        match extract_bundle_tokens_once(client).await {
            Ok((tokens, version)) => {
                save_cached_bundle(&CachedBundle {
                    bundle_version: version,
                    app_id: tokens.app_id.clone(),
                    secrets: tokens.secrets.clone(),
                    private_key: tokens.private_key.clone(),
                    fetched_at: now_unix(),
                });
                return Ok(tokens);
            }
            Err(e) => {
                log::warn!(
                    "[Bundle] Extraction attempt {}/{} failed: {}",
                    attempt,
                    attempts,
                    e
                );
                last_err = Some(e);
                // Back off before the next attempt. The attempts used to fire
                // back-to-back, so a brief network hiccup (DNS blip, dropped
                // connection, captive-portal redirect) failed all of them in a
                // few ms — the retries were effectively useless. A short growing
                // delay gives a transient failure time to clear.
                if attempt < attempts {
                    tokio::time::sleep(Duration::from_millis(600 * attempt as u64)).await;
                }
            }
        }
    }
    Err(last_err
        .unwrap_or_else(|| ApiError::BundleExtractionError("bundle extraction failed".into())))
}

/// Background refresh: cheaply re-check the current bundle version. If Qobuz
/// rotated the bundle, re-extract (and re-cache) the new secrets and return
/// them; if unchanged, just bump the cache freshness timestamp and return
/// `None`. Never blocks the UI — call from a spawned task.
pub async fn refresh_bundle_if_changed(
    client: &Client,
    cached_version: &str,
) -> Option<BundleTokens> {
    let (_, version) = fetch_bundle_url(client).await.ok()?;
    if version == cached_version {
        if let Some(mut c) = load_cached_bundle() {
            c.fetched_at = now_unix();
            save_cached_bundle(&c);
        }
        log::debug!("[Bundle] Background check: version {} unchanged", version);
        return None;
    }
    log::info!(
        "[Bundle] Background check: version changed {} -> {}, re-extracting",
        cached_version,
        version
    );
    extract_and_cache_bundle_tokens(client).await.ok()
}
