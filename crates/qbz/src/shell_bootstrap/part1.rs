use crate::*;

pub(crate) fn dispatch(command: AppCommand) {
    log::info!("[qbz-slint] AppCommand::{} dispatched", command.id());
}

/// Per-user shell wiring shared by the online and offline session entries.
/// None of it requires a Qobuz session: local library DB binding (+ mixtape
/// migrations), per-user pref stores, system tray and media controls.
/// Returns the tray settings snapshot for the UI seeding.
pub(crate) fn init_shell_for_user(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    user_id: u64,
) -> tray_settings::TraySettings {
    // Bind the local library DB to this user (folders / playlist
    // settings live in the per-user library.db).
    library_db::set_user(user_id);

    // Run the Mixtapes & Collections schema migrations against the same
    // per-user library.db (the mixtape tables live in that file). Mirrors
    // the Tauri build's session_lifecycle.rs `run_mixtape_migrations`.
    // Best-effort: log on error, never block shell entry.
    library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            if let Err(e) = qbz_mixtape::schema::run_mixtape_migrations(conn) {
                log::error!("[qbz-slint] mixtape migrations failed: {e}");
            }
        }))
    });

    // Bind tray settings to this user (per-user tray_settings.db, shared with
    // the Tauri build) and snapshot them to seed the settings UI.
    tray_settings::init_for_user(user_id);
    let tray = tray_settings::get();

    // Bind scrobbler (Last.fm + ListenBrainz) settings to this user (per-user
    // scrobbler_settings.db), then start the scrobble runtime: tokio handle
    // for the source-agnostic now-playing/scrobble fire, LB credential seed
    // from the shared cache, and the offline-queue flush watcher (drains the
    // shared scrobble_queue + listen_queue on every offline -> online edge).
    scrobbler_settings::init_for_user(user_id);
    scrobble::start(tokio::runtime::Handle::current());

    // Restore the persisted player volume so audio starts at the saved level
    // (the poll loop then mirrors it onto NowPlayingState for the slider).
    playback::set_volume(
        runtime.clone(),
        weak.clone(),
        tokio::runtime::Handle::current(),
        crate::ui_prefs::load().volume,
    );

    // Bind "My Qoqobuz" nav branding (custom label + icon) to this user
    // (per-user myqbz_branding.json). Seeded into MyQbzBrandingState by the
    // caller so the sidebar row + Settings row reflect the persisted values.
    myqbz_prefs::init_for_user(user_id);

    // Bind per-collection DETAIL view-prefs (toolbar viewMode/sort/filter) to
    // this user (per-user collection_view_prefs.json). Restored on collection
    // open, cleared on delete (spec 12 §18).
    myqbz_view_prefs::init_for_user(user_id);

    // Create the system tray from this user's persisted settings (gated by
    // enable_tray). Reflects the chosen icon variant. On Linux the ksni
    // service runs on its own thread; macOS/Windows are no-ops until the
    // tray-icon slice lands.
    tray::init(
        runtime.clone(),
        weak.clone(),
        tokio::runtime::Handle::current(),
        tray.tray_icon_theme.clone(),
        tray.enable_tray,
    );

    // System media controls — MPRIS on Linux (publishes DesktopEntry so GNOME
    // shows the app icon), SMTC/MediaRemote on macOS/Windows. Independent of
    // the tray; pushes metadata/state from the playback paths.
    media_controls::init(
        runtime.clone(),
        weak.clone(),
        tokio::runtime::Handle::current(),
    );

    tray
}

/// Background-load the Audio + Playback settings into the Settings page —
/// store reads and device enumeration are blocking and fully local. Shared
/// by the online and offline session entries.
pub(crate) fn spawn_settings_snapshot_load(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    settings_ctx: Arc<settings::SettingsCtx>,
) {
    tokio::spawn(async move {
        // Seed the local device-cap cache (#638 fix 3) BEFORE the snapshot
        // build so the Settings "Detected device limit" row is filled on
        // first open and the first governed play resolves against the cap.
        // Instant no-op while the toggle is off (the default).
        settings::refresh_device_cap(&settings_ctx, &weak).await;
        let ctx_for_load = settings_ctx.clone();
        match tokio::task::spawn_blocking(move || settings::load_snapshot(&ctx_for_load)).await {
            Ok(snap) => {
                let _ = weak.upgrade_in_event_loop(move |w| {
                    settings::apply_snapshot(&w, snap);
                });
            }
            Err(e) => log::error!("[qbz-slint] settings load task failed: {e}"),
        }
        // Bit-perfect (ALSA + hw) forces local volume to 100% at startup so
        // the bar reflects unity gain before Settings is ever opened. No-op
        // otherwise (and while controlling a peer). Mirrors Tauri.
        settings::apply_startup_bitperfect_volume(&settings_ctx, &runtime, &weak).await;
    });
}

