//! Per-user store activation, shared by the browser-login and
//! saved-session-restore paths (identical sequencing in both).

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

/// Bring up the per-user offline cache + every per-user store, then flip the
/// D4 valid verdict and end any unauthenticated offline session. Called after
/// `runtime.activate(user_id)` on both the browser-login and saved-session
/// paths.
pub(super) async fn activate_per_user_stores<A>(runtime: &AppRuntime<A>, user_id: u64)
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    // Bring up the per-user offline cache (shared index.db + library.db with Tauri).
    crate::offline::activate(user_id).await;
    crate::offline_cache::load_cached_ids().await;

    // Offline-MODE per-user binding (after offline::activate so the purge
    // consumer can reach the cache), then the D4 valid verdict and the D2
    // recovery: a successful login ends any unauthenticated offline session.
    if let Some(dir) = crate::offline_mode::user_data_dir(user_id) {
        crate::offline_mode::init_for_user(&dir);
        crate::fav_cache::init_for_user(&dir);
        // Reco-scoped "Not interested" dismissal store (reco rails only).
        crate::reco_dismiss::init_for_user(&dir);
        // Recommendation event store (shared events.db with Tauri). Train
        // after init (off-thread) so the seeds reflect this session's events.
        crate::reco::init_for_user(&dir);
        crate::reco::train_async();
        // External-recommendations resolution + per-week Weekly cache (4th
        // Discover tab). MUST init here on the ONLINE path too — it was only
        // wired into the offline entry, so online the reco cache was never
        // bound: every open re-resolved from scratch and the Weekly per-week
        // cache never persisted (that is why the Weekly rows kept vanishing).
        crate::external_reco::init_for_user(&dir);
        // Playlist Suggested Songs: open the per-user artist-vector store on
        // the core (the suggestions engine reads/writes it).
        if let Ok(store) = qbz_reco::ArtistVectorStore::open_at(&dir) {
            runtime.core().set_artist_vectors(store).await;
        }
        crate::discover_prefs::init_for_user(&dir);
        crate::artist_blacklist::init_for_user(&dir);
        crate::pinned::init_for_user(&dir);
        crate::local_favorites::init_for_user(&dir);
        // Intelligent Search (cache + ranking), seeded from the persisted pref.
        crate::search_service::init(&dir, crate::ui_prefs::load().intelligent_search);
        // Session persistence (queue + playback): open the per-user session.db
        // and seed the persist/resume gates from the playback prefs.
        crate::session_persist::init_for_user(&dir);
    }
    crate::offline_mode::subscription_mark_valid();
    crate::offline_mode::engine().set_offline_session(false);
}
