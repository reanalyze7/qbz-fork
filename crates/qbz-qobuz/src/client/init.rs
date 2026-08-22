use std::sync::Arc;

use super::QobuzClient;
use crate::bundle;
use crate::error::Result;

impl QobuzClient {
    /// Initialize client by extracting bundle tokens.
    ///
    /// Warm start: if cached tokens exist, use them immediately so the UI never
    /// blocks on Qobuz's (sometimes very slow) ~7 MB bundle download, then
    /// refresh in the background — re-downloading only if Qobuz rotated the
    /// bundle version. Cold start (first run or after a cache wipe): fetch now,
    /// bounded by a per-request timeout + a small retry so a slow/dead CDN can't
    /// hang forever.
    ///
    /// Returns `true` if it served cached tokens (warm), `false` if it had to do
    /// a live extraction (cold) — callers can use this to drive a "connecting"
    /// UI only when it actually matters.
    pub async fn init(&self) -> Result<bool> {
        if let Some(cached) = bundle::load_cached_bundle() {
            let version = cached.bundle_version.clone();
            log::info!("[Bundle] Using cached tokens (version {})", version);
            *self.tokens.write().await = Some(cached.into());

            // Cache reads are never gated, but the background refresh is a
            // network request — gate it once before cloning the client into
            // the spawned task, skipping the refresh instead of failing the
            // warm start.
            match self.http() {
                Ok(client) => {
                    let client = client.clone();
                    let tokens_arc = Arc::clone(&self.tokens);
                    tokio::spawn(async move {
                        if let Some(fresh) =
                            bundle::refresh_bundle_if_changed(&client, &version).await
                        {
                            *tokens_arc.write().await = Some(fresh);
                            log::info!("[Bundle] Background refresh applied rotated tokens");
                        }
                    });
                }
                Err(_) => {
                    log::info!("[Bundle] Offline mode - skipping background bundle refresh");
                }
            }
            return Ok(true);
        }

        log::info!("[Bundle] No cached tokens, extracting from Qobuz...");
        // Cold start: a live bundle fetch is a network request — gated on
        // purpose so an offline cold start fails fast instead of waiting out
        // the network timeouts.
        let tokens = bundle::extract_and_cache_bundle_tokens(self.http()?).await?;
        *self.tokens.write().await = Some(tokens);
        Ok(false)
    }
}
