use crate::*;

/// Startup page = "where you left off": restore the last SAFE top-level view
/// (online only — the offline entry keeps its D12 LocalLibrary root). Home
/// was already loaded; if a different view is remembered, re-root the nav
/// history there and apply_entry it.
pub(crate) async fn es_restore_startup_page(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
) {
    // Startup page = "where you left off": restore the last SAFE top-level view
    // (online only — the offline entry keeps its D12 LocalLibrary root). Home
    // was loaded just above; if a different view is remembered, re-root the nav
    // history there (on the UI thread, like the offline path) and apply_entry it
    // — which loads the view's data, NOT a blank set_view (the Tauri precedent).
    {
        let prefs = crate::ui_prefs::load();
        // Crash-chain gate: at level >=2 the persisted view restore was
        // already reset by `arm_startup_probe` (last_nav "{}" / last_view
        // "home"), so there is nothing valid to restore — skip the block
        // explicitly, tell the user what happened, and stay on Home.
        if crash_chain_level() >= 2 {
            log::warn!("[crash-chain] persisted view restore skipped (recovery)");
            let _ = weak.upgrade_in_event_loop(|w| {
                crate::toast::info(
                    &w,
                    qbz_i18n::t(
                        "Qoqobuz recovered from repeated startup crashes — some restored state was reset",
                    ),
                );
            });
        } else if prefs.startup_page == "remember" {
            // Legacy top-level fallback (id-free surfaces) — the only thing that
            // can be restored offline (these load from local/offline data).
            let legacy = |key: &str| match key {
                "favorites" => Some(nav::NavEntry::Favorites { tab: "tracks".to_string() }),
                "local-library" => Some(nav::NavEntry::LocalLibrary {
                    tab: local_library::LibTab::Albums.tab_id().to_string(),
                }),
                "mixtapes" => Some(nav::NavEntry::Mixtapes),
                "collections" => Some(nav::NavEntry::Collections),
                _ => None,
            };
            // Online: restore the EXACT last view from the full JSON entry
            // (album/artist/playlist/mix/label/… re-fetched by id), falling back
            // to the legacy top-level key. Offline: only the legacy fallback, so
            // a remembered online detail view doesn't fail-load behind the
            // offline gate (it keeps the D12 LocalLibrary/Home root).
            let entry = if crate::offline_mode::engine().is_offline() {
                legacy(&prefs.last_view)
            } else {
                prefs
                    .last_nav
                    .as_deref()
                    .and_then(|j| serde_json::from_str::<nav::NavEntry>(j).ok())
                    .or_else(|| legacy(&prefs.last_view))
            };
            // Home was loaded above; only re-root when a different view is
            // remembered. apply_entry loads the view's data (re-fetch by id); a
            // stale id surfaces its own "couldn't load" toast.
            if let Some(entry) = entry.filter(|e| !matches!(e, nav::NavEntry::Home)) {
                let root_entry = entry.clone();
                let _ = weak.upgrade_in_event_loop(move |_w| {
                    nav::reset_root(root_entry);
                });
                apply_entry(
                    entry,
                    &runtime,
                    &weak,
                    &tokio::runtime::Handle::current(),
                    &image_cache,
                );
            }
        }
    }
}
