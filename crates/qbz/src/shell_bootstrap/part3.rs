// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for.
use crate::*;

/// Reveal the shell and load the Discover / Home view with real data,
/// then kick off cached artwork downloads.
pub(crate) async fn enter_shell(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    settings_ctx: Arc<settings::SettingsCtx>,
    session: auth::SessionInfo,
) {
    let tray = init_shell_for_user(&runtime, &weak, session.user_id);

    // Deep links (argv capture / warm D-Bus OpenUrl) may now drain: the
    // session is active and the AppWindow exists. Bound here at the top; the
    // pending URL itself is dispatched at the very END of this function so
    // the startup-page/view restore below can't re-root over the deep link.
    deep_link::bind_shell_ctx(
        runtime.clone(),
        weak.clone(),
        tokio::runtime::Handle::current(),
        image_cache.clone(),
    );

    let _ = weak.upgrade_in_event_loop(move |w| {
        let state = w.global::<SessionState>();
        state.set_user_name(session.display_name.into());
        state.set_subscription(session.subscription.into());
        // A successful login means a previous session now exists; clear any
        // stale boot restore error from the login screen.
        let offline_state = w.global::<OfflineState>();
        offline_state.set_has_previous_session(true);
        offline_state.set_login_error("".into());
        // Reset the browser sign-in narration for the next visit to the
        // login screen (logout → login).
        let login_state = w.global::<LoginState>();
        login_state.set_phase(0);
        login_state.set_error("".into());
        seed_tray_appearance(&w, &tray);
        // Seed the My QBZ branding (label + icon) from the per-user store so
        // the sidebar row + Settings row paint the custom values immediately.
        myqbz_prefs::seed(&w);
        // Seed the Discover configurator descriptor lists so the prefs-driven
        // render loop has order/visibility data before the first apply_home.
        discover_prefs::seed(&w);
        // Seed the Pinned section (Home / For You) from the per-user pinned
        // store — bound by perform_login / restore before this closure runs.
        pinned_section::rebuild_pinned(&w);
        w.global::<HomeState>().set_loading(true);
        w.set_screen(AppScreen::Shell);
    });

    // Start the playback poll loop — it runs for the app lifetime,
    // ticking position/progress onto NowPlayingState and auto-advancing
    // the queue on track end. Safe to start once per shell entry.
    playback::start_poll_loop(runtime.clone(), weak.clone(), tokio::runtime::Handle::current());
    // Bind the exit context so the window close handlers can flush a final
    // session snapshot before the loop quits (idempotent).
    session_persist::bind_exit_ctx(runtime.clone(), tokio::runtime::Handle::current());

    // Load the sidebar playlists list.
    load_sidebar_playlists(runtime.clone(), weak.clone(), &tokio::runtime::Handle::current());

    // Warm the shared favorite-track cache so track rows can show the
    // correct heart state from their first paint (album / artist / search
    // / playlist / mix / favorites / queue all read it). The disk seed
    // already ran at session activation (fav_cache::init_for_user); this
    // refreshes from the network and writes the fresh set back — skipped
    // while offline, where the disk seed is the truth.
    {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            match runtime.core().favorite_track_ids().await {
                Ok(ids) => {
                    // set_all mirrors to disk (blocking rusqlite) — keep it
                    // off the async worker.
                    let _ = tokio::task::spawn_blocking(move || fav_cache::set_all(ids)).await;
                }
                Err(e) => log::warn!("[qbz-slint] favorite cache load failed: {e}"),
            }
        });
    }

    // Same for favorite ALBUMS — seeds fav_cache so the album header heart is
    // correct from first open without visiting the Favorites view.
    {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            let ids = favorites::favorite_album_ids(&runtime).await;
            let _ = tokio::task::spawn_blocking(move || fav_cache::set_all_albums(ids)).await;
        });
    }

    // Seed the per-artist library index so the ArtistPage catalog/library toggle
    // can decide (O(1)) whether the user has items for that artist. Favorites-
    // only (tracks + albums), once per session, off the UI thread.
    {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            crate::library_by_artist::seed(&runtime).await;
        });
    }

    // Same for followed ARTISTS — the Pinned carousel's artist follow chip
    // seeds from fav_cache at build time (its only build-time consumer), and
    // the pinned model is built BEFORE this warm lands: re-seed any already-
    // built artist rows once the fresh set arrives (walk > rebuild_pinned: no
    // model swap, no artwork-job re-dispatch, no flicker).
    {
        let runtime = runtime.clone();
        let weak = weak.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            match runtime.core().favorite_artist_ids().await {
                Ok(ids) => {
                    let _ =
                        tokio::task::spawn_blocking(move || fav_cache::set_all_artists(ids)).await;
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        let pm = w.global::<PinnedState>().get_items();
                        for i in 0..pm.row_count() {
                            if let Some(mut it) = pm.row_data(i) {
                                if it.kind == "artist" {
                                    let following = it
                                        .artist
                                        .id
                                        .parse::<u64>()
                                        .map(|id| fav_cache::is_artist_favorite(id))
                                        .unwrap_or(false);
                                    if it.artist.following != following {
                                        it.artist.following = following;
                                        pm.set_row_data(i, it);
                                    }
                                }
                            }
                        }
                        // External-reco artist rows (Discover > Recommendations):
                        // same in-place re-seed — the rows may already be painted
                        // from the results blob before this warm lands (their
                        // build-time fav_cache seed was stale/empty then).
                        let reco = w.global::<ExternalRecoState>();
                        for model in [
                            reco.get_rec_artists_common(),
                            reco.get_rec_artists_recent(),
                            reco.get_top_artists(),
                        ] {
                            for i in 0..model.row_count() {
                                if let Some(mut it) = model.row_data(i) {
                                    let following = it
                                        .id
                                        .parse::<u64>()
                                        .map(|id| fav_cache::is_artist_favorite(id))
                                        .unwrap_or(false);
                                    if it.following != following {
                                        it.following = following;
                                        model.set_row_data(i, it);
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => log::warn!("[qbz-slint] favorite artists warm failed: {e}"),
            }
        });
    }

    // Load Audio + Playback settings into the Settings page in the
    // background — store reads and device enumeration are blocking.
    spawn_settings_snapshot_load(runtime.clone(), weak.clone(), settings_ctx.clone());

    // Load the genre-filter parents + persisted selection, then seed
    // the popup state. Done before the discover load so the first
    // fetch honors a remembered genre selection.
    genre_filter::load_parents(&runtime).await;
    let _ = weak.upgrade_in_event_loop(|w| {
        genre_filter::apply_state(&w);
    });

    reload_home(&runtime, &weak, &image_cache, "home".to_string()).await;

    // Session persistence: restore the last queue + current track PAUSED (gated
    // on `persist_session`). set_queue_with_order emits QueueUpdated so the queue
    // sidebar repaints itself; the now-playing bar reads current_track, so we
    // refresh its metadata explicitly. No audio is loaded — playback stays
    // stopped until the user hits play (Phase B then seeks to the saved
    // position when `resume_playback_position` is on).
    if crash_chain_level() >= 3 {
        // Crash-chain level >=3: two consecutive starts died even after the
        // view-restore reset — bypass the queue restore for THIS boot only
        // (the persisted queue stays on disk; a healthy boot restores it).
        log::warn!("[crash-chain] session-persist queue restore bypassed this boot");
    } else if session_persist::restore(&runtime).await {
        playback::refresh_now_playing_meta(&runtime, &weak).await;
        // Repaint the queue sidebar/list — set_queue_with_order emits
        // QueueUpdated, but the queue UI repaints from explicit refreshes.
        playback::refresh_sidebar(true);
        // Seed the seek bar + timers to the resume position so they show it
        // immediately (refresh_now_playing_meta above reset them to 0; the poll
        // loop only catches up once playback starts). Peeks — the actual resume
        // still fires on first play.
        //
        // KNOWN ISSUE / NEEDS WORK: this seed does NOT visibly stick — at rest
        // the bar + timer still read 0:00 and only jump to the resume position
        // once the user presses play (the audio resume itself works correctly).
        // Something repaints NowPlayingState position/progress back to 0 after
        // this runs (a later refresh_now_playing_meta closure, the poll loop's
        // idle tick reporting position 0 while no audio is loaded, or the bar
        // binding not reflecting a paused non-loaded position). Left as-is on
        // purpose — revisit the pre-play seek-bar seed for paused restore.
        let resume_pos = session_persist::pending_resume_position();
        if resume_pos > 0 {
            if let Some(track) = runtime.core().current_track().await {
                let dur = track.duration_secs;
                let _ = weak.upgrade_in_event_loop(move |w| {
                    playback::seed_seek_display(&w, resume_pos, dur);
                });
            }
        }
    }

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
                        "QBZ recovered from repeated startup crashes — some restored state was reset",
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

    // Seed the favorites tab counts so the badges are ready before the
    // user opens each tab (they otherwise only count on first visit).
    let counts = favorites::load_counts(&runtime).await;
    let _ = weak.upgrade_in_event_loop(move |w| {
        favorites::apply_counts(&w, counts);
    });

    // XDG deep link: drain a pending Qobuz URL LAST — after the startup-page
    // / view-restore block above, so the restore can't re-root over the deep
    // link (the navigation lands on top of whatever was restored). Session
    // active, AppWindow alive: no readiness sleep needed. Nothing pending =>
    // no-op. (Offline entries never reach here — enter_shell_offline keeps
    // the URL pending; navigation needs the API.)
    deep_link::drain_pending();
}

