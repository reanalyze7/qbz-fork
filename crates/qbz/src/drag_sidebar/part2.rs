use crate::*;

/// After a successful playlist rename, reload the sidebar but HOLD the
/// optimistic name until the fetched data agrees with it. Replaces a plain
/// `load_sidebar_playlists` in the rename arms: Qobuz's playlist/list can
/// lag read-after-write, so the naive reload overwrote the optimistic patch
/// with the stale server name (the visible whiplash: new name → old name
/// until a later manual refresh). Bounded retries with linear backoff; the
/// sidebar shows its loading shimmer while reconciling. Local playlists
/// converge on the first pass (the library.db read is already fresh).
pub(crate) fn reconcile_sidebar_after_rename(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    id: String,
    expected: String,
) {
    const MAX_ATTEMPTS: u32 = 6;
    let _ = weak.upgrade_in_event_loop(|w| sidebar::set_loading(&w, true));
    handle.spawn(async move {
        let expected = expected.trim().to_string();
        let numeric = id.parse::<u64>().ok();
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let data = sidebar::load(&runtime).await;
            let fetched: Option<String> = if let Some(n) = numeric {
                data.playlists
                    .iter()
                    .find(|p| p.id == n)
                    .map(|p| p.name.trim().to_string())
            } else {
                data.local_playlists
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| p.name.trim().to_string())
            };
            // A missing entry counts as consistent (deleted elsewhere / not
            // listed): apply as-is rather than spin on a row that is gone.
            let consistent = fetched.as_deref().map(|n| n == expected).unwrap_or(true);
            if consistent || attempt >= MAX_ATTEMPTS {
                if !consistent {
                    log::warn!(
                        "[playlist-rename] list still shows the old name after {attempt} attempts; keeping the optimistic name visible"
                    );
                }
                let _ = weak.upgrade_in_event_loop(move |w| {
                    sidebar::apply(&w, data);
                    refresh_sidebar_covers(&w);
                    if !consistent {
                        sidebar::rename_entry(&w, &id, &expected);
                    }
                });
                break;
            }
            // Stale server: keep the optimistic name visible and retry.
            let id2 = id.clone();
            let expected2 = expected.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                sidebar::rename_entry(&w, &id2, &expected2);
            });
            tokio::time::sleep(std::time::Duration::from_secs(attempt as u64)).await;
        }
    });
}

/// (Re)spawn the per-playlist micro-collage cover downloads for the
/// current `SidebarState.entries`. Called after any rebuild that replaces
/// the rows (load / toggle / move / sort / search), since `set_row_data`
/// resets the decoded cover images. Each completion updates only its own
/// row (see artwork.rs), and the shared image cache means already-fetched
/// covers resolve from disk without a re-download.
pub(crate) fn refresh_sidebar_covers(window: &AppWindow) {
    if let Some(cache) = artwork::shared_cache() {
        let (qobuz_jobs, local_jobs) = sidebar::artwork_jobs(window);
        if !qobuz_jobs.is_empty() {
            artwork::spawn_loads(qobuz_jobs, window.as_weak(), cache.clone());
        }
        // LOCAL playlist collage covers are file paths — route them through
        // the local loader (http loader would miss them).
        if !local_jobs.is_empty() {
            artwork::spawn_local_loads(local_jobs, window.as_weak(), cache);
        }
    }
}

