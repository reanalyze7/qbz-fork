use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_app::playback_driver;

use crate::adapter::DaemonAdapter;
use crate::config::QbzdConfig;
use crate::paths::ProfileRoots;

use super::session::{
    is_auth_rejection, latch_auth_error, latch_undecryptable_token, new_shared, restore_activate,
    set_needs_auth, spawn_auth_retry,
};
use super::BootedRuntime;

/// Steps 6-9 of §8.1: open the daemon-root stores, compose the runtime with the
/// two NORMATIVE substitutions (`with_audio_settings` + `activate_at`), and
/// restore the saved session per the §6.2 clearing taxonomy.
pub(super) async fn boot(
    roots: &ProfileRoots,
    cfg: &QbzdConfig,
    warn_count: usize,
) -> Result<BootedRuntime, String> {
    // 6.+7. stores + runtime composition. The two substitutions (01 §2.2):
    //   - with_audio_settings, NOT AppRuntime::new (which hardcodes the
    //     desktop-global AudioSettingsStore — shell.rs:87-101);
    //   - activate_at (below), NOT activate (which resolves desktop
    //     UserDataPaths — shell.rs:195-203).
    // Everything routes through the T2 daemon roots.
    let store = qbz_audio::settings::AudioSettingsStore::new_at(&roots.data)?; // settings.rs:263
    let settings = store.get_settings()?;
    let (adapter, _rx) = DaemonAdapter::new();
    let bus = adapter.sender();
    let runtime = Arc::new(AppRuntime::with_audio_settings(
        adapter,
        settings.output_device.clone(),
        settings,
        None,
    )); // shell.rs:64

    // Offline-tolerant (§8.1-8): a network failure here still leaves a locally
    // usable core; a missing DAC is likewise non-fatal (Player starts deviceless
    // and retries with backoff — never the spotifyd #1097 crash-exit).
    if let Err(e) = runtime.init().await {
        log::warn!("core init did not complete (continuing offline-tolerant): {e}");
    }

    // Playlist recommendations (CONSOLE): open the per-user artist-vector store
    // at the DAEMON root — mirrors qbz/src/auth.rs:145-149, but slint-free and
    // session-independent. The store is a CACHE the suggestions engine reads/
    // writes; vectors are built on demand from MusicBrainz + Qobuz, so this
    // needs no listening history. Best-effort: a failed open leaves
    // `generate_playlist_suggestions` working un-cached (artist_vectors = None).
    if let Ok(store) = qbz_reco::ArtistVectorStore::open_at(&roots.data) {
        runtime.core().set_artist_vectors(store).await;
    }

    let shared = new_shared(cfg);
    if let Ok(mut s) = shared.lock() {
        s.startup_warnings = warn_count as u32;
    }

    // 8. credential restore per the §6.2 taxonomy (mirrors qbz/src/auth.rs:
    //    215-230): clear the token ONLY on explicit auth rejection; KEEP it on
    //    every network-class failure (clearing on transient errors is the
    //    documented boot-token-loss bug class).
    let auth_retry = match qbz_credentials::load_oauth_token_at(&roots.config)? {
        None => {
            set_needs_auth(&shared, None);
            // `None` covers both "no token saved" and "token saved but this
            // process cannot decrypt it" — the decrypt failure is swallowed by
            // design so a broken file can never abort boot. Tell them apart
            // here, or `status` reports "not logged in / last error: none" and
            // the real cause stays buried in the log.
            if qbz_credentials::oauth_token_file_present_at(&roots.config) {
                latch_undecryptable_token(&shared);
            }
            None
        }
        Some(token) => {
            // Register before the token can reach any log line (§6.3).
            qbz_log::register_secret(token.clone());
            match runtime.core().login_with_token(&token).await {
                Ok(session) => {
                    restore_activate(&runtime, &shared, roots, session, &token).await?;
                    // 9½. session restore (queue/position) PAUSED: the daemon's
                    //     session store IS its queue persistence, so a restart
                    //     comes back with the queue armed but not auto-playing.
                    playback_driver::restore_session_paused(runtime.as_ref()).await;
                    None
                }
                Err(e) if is_auth_rejection(&e) => {
                    qbz_credentials::clear_oauth_token_at(&roots.config)?;
                    latch_auth_error(&shared, &e);
                    set_needs_auth(&shared, Some(e));
                    None
                }
                Err(e) => {
                    // network-class: KEEP token, stay Restoring, retry w/ backoff.
                    log::warn!("session restore deferred (network-class): {e}");
                    // 01 §9.3: a real network-class outcome — latch `network.online`
                    // false so `/api/status` reflects it until a retry succeeds.
                    if let Ok(s) = shared.lock() {
                        s.set_network_online(false);
                    }
                    Some(spawn_auth_retry(
                        runtime.clone(),
                        shared.clone(),
                        roots.clone(),
                    ))
                }
            }
        }
    };

    Ok(BootedRuntime {
        runtime,
        shared,
        bus,
        auth_retry,
    })
}
