use crate::*;

/// Offline session entry — "Start offline" on the login screen (spec §4.1).
///
/// Mirrors the subset of `enter_shell` + Tauri's `activate_offline_session`
/// that works without a Qobuz session: session scaffolding, local library,
/// offline cache, per-user pref stores, tray/media controls, the playback
/// poll loop and the settings snapshot. Everything Qobuz-bound is skipped
/// (sidebar playlists, favorites warm, genre filter, home/discover load) —
/// the engine's gate refuses those calls anyway (D3).
pub(crate) async fn enter_shell_offline(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    settings_ctx: Arc<settings::SettingsCtx>,
) -> Result<(), String> {
    // No previous session → the GUEST profile, user 0 (#553; restores the
    // Tauri fallback this port used to refuse). The guest builds a real
    // per-user store set (local library, mixtapes, prefs) under users/0;
    // the first successful login ADOPTS it (AppRuntime::activate renames
    // users/0 to the real id when that account has no profile here yet).
    // `activate_offline` below resolves the same id — 0 is never persisted
    // as the last-user marker, so it never masquerades as a real session.
    let user_id = qbz_app::user_data::UserDataPaths::load_last_user_id().unwrap_or(0);
    if user_id == 0 {
        log::info!(
            "[qbz-slint] offline shell: no previous session — entering the guest profile (user 0)"
        );
    }

    // Session scaffolding at the last user (session store, runtime state).
    runtime.activate_offline().await?;

    // Offline cache (shared index.db + library.db) + the in-memory cached-ids
    // set the track rows read. Must precede offline_mode::init_for_user so
    // the subscription purge consumer can reach the cache.
    crate::offline::activate(user_id).await;
    crate::offline_cache::load_cached_ids().await;

    // Local library, per-user pref stores, tray, media controls.
    let tray = init_shell_for_user(&runtime, &weak, user_id);

    // Offline-MODE engine: bind the per-user stores and flag the
    // unauthenticated offline session (D1 — session-scoped, never persisted;
    // a later successful login clears it). The favorites-cache bind seeds
    // hearts from disk — the start-offline gap Tauri never closed.
    if let Some(dir) = crate::offline_mode::user_data_dir(user_id) {
        crate::offline_mode::init_for_user(&dir);
        crate::fav_cache::init_for_user(&dir);
        // Reco-scoped "Not interested" dismissal store (Discover >
        // Recommendations rails only — NOT the blacklist).
        crate::reco_dismiss::init_for_user(&dir);
        // Recommendation event store (shared events.db with Tauri).
        crate::reco::init_for_user(&dir);
        // External-recommendations resolution cache (4th Discover tab).
        crate::external_reco::init_for_user(&dir);
        // Playlist Suggested Songs: open the per-user artist-vector store.
        if let Ok(store) = qbz_reco::ArtistVectorStore::open_at(&dir) {
            runtime.core().set_artist_vectors(store).await;
        }
        crate::discover_prefs::init_for_user(&dir);
        // D-FIX-a: bind the blacklist offline too — Tauri never initialized it
        // in offline mode, so blacklisted artists leaked into offline surfaces.
        crate::artist_blacklist::init_for_user(&dir);
        // Pinned items are local-only and must render offline too.
        crate::pinned::init_for_user(&dir);
        crate::local_favorites::init_for_user(&dir);
        // Intelligent Search (cache + ranking), seeded from the persisted pref.
        // Cached results stay searchable offline; live revalidation no-ops.
        crate::search_service::init(&dir, crate::ui_prefs::load().intelligent_search);
        // Session persistence (queue + playback): open the per-user session.db
        // and seed the persist/resume gates from the playback prefs.
        crate::session_persist::init_for_user(&dir);
    }
    crate::offline_mode::engine().set_offline_session(true);

    {
        let weak = weak.clone();
        let _ = weak.clone().upgrade_in_event_loop(move |w| {
            seed_tray_appearance(&w, &tray);
            myqbz_prefs::seed(&w);
            // Seed the Discover configurator descriptor lists (works offline —
            // the prefs store is per-user and bound at session activation).
            discover_prefs::seed(&w);
            // Seed the Pinned section — pinned items are local-only and must
            // render offline; disk-cached covers still resolve.
            pinned_section::rebuild_pinned(&w);
            // No HomeState loading spinner: the discover load is skipped offline
            // (the gating slice adds the placeholder views).
            w.set_screen(AppScreen::Shell);
            // D12: an offline session lands on LocalLibrary (Home is a blocked
            // placeholder offline). Root the nav history at it so back/forward
            // never lead to a phantom blocked Home.
            nav::reset_root(nav::NavEntry::LocalLibrary {
                tab: local_library::LibTab::Albums.tab_id().to_string(),
            });
            update_nav_flags(&w);
        });
    }
    navigate_local_library(
        runtime.clone(),
        weak.clone(),
        &tokio::runtime::Handle::current(),
        image_cache,
        local_library::LibTab::Albums,
    );

    // Sidebar playlists: offline the load lists the LOCAL playlists plus the
    // MIXED Qobuz playlists with local sidecar content (D11.b) — the Qobuz
    // fetch itself fast-fails at the gate.
    load_sidebar_playlists(runtime.clone(), weak.clone(), &tokio::runtime::Handle::current());

    // Playback poll loop — local/cached playback and queue advance work
    // offline. Same lifetime semantics as the online entry.
    playback::start_poll_loop(runtime.clone(), weak.clone(), tokio::runtime::Handle::current());
    // Bind the exit context so the window close handlers can flush a final
    // session snapshot before the loop quits (idempotent).
    session_persist::bind_exit_ctx(runtime.clone(), tokio::runtime::Handle::current());

    // Load Audio + Playback settings into the Settings page in the
    // background — fully local, same path as the online entry.
    spawn_settings_snapshot_load(runtime.clone(), weak.clone(), settings_ctx.clone());

    log::info!("[qbz-slint] offline session entered for user {user_id}");
    Ok(())
}

