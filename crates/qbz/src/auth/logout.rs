//! `logout`: clear the saved token and tear down every per-user store.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

/// Log out: clear the saved token, deactivate the per-user session, and
/// drop the Qobuz client session.
pub async fn logout<A>(runtime: &Arc<AppRuntime<A>>) -> Result<(), String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let _ = qbz_credentials::clear_oauth_token();
    let _ = runtime.core().logout().await;
    crate::offline::deactivate().await;
    crate::offline_mode::teardown();
    crate::fav_cache::teardown();
    crate::reco_dismiss::teardown();
    crate::reco::teardown();
    runtime.core().clear_artist_vectors().await;
    crate::discover_prefs::teardown();
    crate::artist_blacklist::teardown();
    crate::pinned::teardown();
    crate::local_favorites::teardown();
    crate::search_service::teardown();
    runtime.deactivate().await?;
    log::info!("[qbz-slint] logged out");
    Ok(())
}
