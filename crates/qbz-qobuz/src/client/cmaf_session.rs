use reqwest::StatusCode;

use super::{body_preview, CmafSession, QobuzClient};
use crate::auth::{get_timestamp, sign_session_start};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Ensure we have a valid CMAF session, renewing if expired.
    /// Returns `(session_id, infos)` for use with file/url and key derivation.
    ///
    /// Concurrency note: this method serializes concurrent session
    /// renewals. Without that, two overlapping callers could both see
    /// "no session" on the read side, each POST /session/start, each get
    /// DIFFERENT `infos`, and the second one to finish would overwrite
    /// the first in the cache. Any `get_file_url` response whose wrapped
    /// key was tied to the first session then unwrapped with the second
    /// session's key and blew up with AES-CBC "Unpad Error" — which
    /// manifested as prefetch CMAF failures + downloaded-but-gappy
    /// transitions between offline tracks.
    ///
    /// Fix: a double-checked lock pattern on the write guard. Fast path
    /// uses a read guard; slow path acquires the write guard, re-checks
    /// under exclusive ownership, and only one caller hits the network.
    pub async fn ensure_cmaf_session(&self) -> Result<(String, String)> {
        let now = get_timestamp();

        // Fast path: existing session with > 60s left.
        {
            let guard = self.cmaf_session.read().await;
            if let Some(ref cs) = *guard {
                if cs.expires_at > now + 60 {
                    return Ok((cs.session_id.clone(), cs.infos.clone()));
                }
            }
        }

        // Slow path: take the write lock and re-check. Concurrent callers
        // end up here one at a time; after the first finishes POST
        // session/start, the rest find the freshly-populated cache and
        // return without hitting the network.
        let mut guard = self.cmaf_session.write().await;
        if let Some(ref cs) = *guard {
            if cs.expires_at > now + 60 {
                return Ok((cs.session_id.clone(), cs.infos.clone()));
            }
        }

        // We're the one task that actually starts a session. Back off before
        // the network if the 403 breaker is open (issue #637) — a cached
        // session above is still served; only the network POST is gated.
        self.forbidden_guard()?;
        log::info!("[CMAF] Starting new session");
        let timestamp = get_timestamp();
        let sig = sign_session_start(timestamp);

        let url = endpoints::build_url(paths::SESSION_START);
        let response = self
            .http()?
            .post(&url)
            .headers(self.authenticated_headers().await?)
            .form(&[
                ("profile", "qbz-1"),
                ("request_ts", &timestamp.to_string()),
                ("request_sig", &sig),
            ])
            .send()
            .await?;

        let status = response.status();
        // Feed the breaker: a 403 here counts toward opening it; success resets.
        self.note_forbidden_status(status);
        if !status.is_success() {
            if status == StatusCode::FORBIDDEN {
                let preview = body_preview(response).await;
                log::warn!("[CMAF] session/start 403{}", preview);
                return Err(ApiError::Forbidden(preview));
            }
            return Err(ApiError::ApiResponse(format!(
                "session/start failed with status {}",
                status
            )));
        }

        let resp: SessionStartResponse = response.json().await?;
        let infos = resp.infos.unwrap_or_default();
        log::info!(
            "[CMAF] Session started: id={}..., expires_at={}",
            &resp.session_id[..resp.session_id.len().min(8)],
            resp.expires_at
        );

        let session_id = resp.session_id.clone();
        let infos_clone = infos.clone();

        *guard = Some(CmafSession {
            session_id: resp.session_id,
            infos,
            expires_at: resp.expires_at,
        });

        Ok((session_id, infos_clone))
    }
}
