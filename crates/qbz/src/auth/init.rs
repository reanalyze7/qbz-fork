//! Gated cold bundle-token init + auth-rejection classification.

use qbz_core::FrontendAdapter;

/// Make sure the Qobuz client holds bundle tokens before a sign-in call.
///
/// The sign-in POSTs are gate-exempt, but `try_init_api`'s cold bundle
/// fetch is a network SERVICE request and stays gated on purpose. When an
/// unauthenticated offline session holds the gate closed (sign-in from the
/// offline shell's badge flyout / recovery path), lift the session flag
/// ONLY around this init and put it back immediately after, success or
/// failure — the offline session must end exclusively on a COMPLETED
/// login (the callers' success paths), never as a side effect of merely
/// starting an attempt. No-op in the normal case: an offline session boots
/// from cached bundle tokens, so the client is already initialized and the
/// flag is never touched.
pub(super) async fn ensure_api_initialized<A>(core: &qbz_core::QbzCore<A>) -> Result<(), String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    if core.is_api_initialized().await {
        return Ok(());
    }
    let offline_engine = crate::offline_mode::engine();
    let lifted = offline_engine.status().offline_session;
    if lifted {
        log::info!(
            "[qbz-slint] cold bundle init with an offline session active — lifting the flag for the init only"
        );
        offline_engine.set_offline_session(false);
    }
    let result = core.try_init_api().await.map_err(|e| e.to_string());
    if lifted {
        offline_engine.set_offline_session(true);
    }
    result
}

/// True only for an EXPLICIT auth rejection from Qobuz: a 401 on the token
/// login (`AuthenticationError`) or an ineligible-account verdict. Network
/// failures, the offline gate, 5xx, rate limiting, parse errors and unknown
/// statuses (`ApiResponse`) all return false — on those the saved token must
/// be KEPT (spec §4.1 D1: the boot token-clearing bug).
pub(super) fn is_auth_rejection(error: &qbz_core::CoreError) -> bool {
    matches!(
        error,
        qbz_core::CoreError::Api(
            qbz_qobuz::ApiError::AuthenticationError(_) | qbz_qobuz::ApiError::IneligibleUser
        )
    )
}
