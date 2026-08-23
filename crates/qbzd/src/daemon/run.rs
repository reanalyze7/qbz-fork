use std::sync::Arc;

use qbz_app::settings::daemon_prefs;
use qbz_app::playback_driver;

use crate::config::QbzdConfig;
use crate::lock::InstanceLock;
use crate::paths::ProfileRoots;

use super::bind::{diagnose_lock, diagnose_port_conflict, resolve_bind_addr, wait_for_signal};
use super::boot::boot;
use super::shutdown::{shutdown, RunHandles};
use super::subsystems;

/// `qbzd run` — boot the daemon in the foreground, park on signals, shut down
/// gracefully. Returns the process exit code (0 = clean shutdown). `warns` are
/// the unknown-key warnings surfaced by [`QbzdConfig::load`] in `main`.
pub async fn run(roots: ProfileRoots, cfg: QbzdConfig, warns: Vec<String>) -> Result<i32, String> {
    // 1. argv parse happened in main(). 2. logging:
    qbz_log::install(&cfg.log.level);
    // 3. config: surface unknown-key warnings (they never abort — D14).
    for w in &warns {
        log::warn!("[config] unknown key: {w}");
    }
    // 4. instance lock on the DATA ROOT, taken BEFORE any port bind (§8.3): it,
    //    not the port, protects the single-device_uuid / single-session.db
    //    invariants. A second daemon on the same root is diagnosed → exit 3.
    let _lock = InstanceLock::acquire(&roots.data).map_err(diagnose_lock)?;
    // 5. port bind + foreign-occupant diagnosis — STATELESS, so it runs BEFORE
    //    stores (6) and runtime composition (7) per the §8.1 order. On a bind
    //    conflict the occupant is probed with GET /api/ping: a qbzd answer means
    //    a stale foreign root (the lock said this root was free), anything else
    //    the §2.2 "another process" copy. The socket is bound here but not served
    //    until step 11 — connections queue in the listen backlog through boot.
    let bind_addr = resolve_bind_addr(&cfg)?;
    let bound = match crate::api::bind(bind_addr) {
        Ok(b) => b,
        Err(crate::api::BindError::AddrInUse(addr)) => return Err(diagnose_port_conflict(addr)),
        Err(crate::api::BindError::Other(msg)) => {
            return Err(format!(
                "error: could not bind the control API on {bind_addr}: {msg}\n  → check [server] bind/port in ~/.config/qbzd/qbzd.toml"
            ));
        }
    };
    if !bind_addr.ip().is_loopback() {
        // FB6: the default bind is now 0.0.0.0 — LAN-first posture (Sonos/
        // Chromecast parity), not a misconfiguration. One INFO line, not a
        // stderr warning; loopback binds stay silent.
        log::info!("{}", crate::cli::copy::lan_posture_note(&bind_addr.to_string()));
    }

    // 6.-9. compose stores + runtime + restore credentials + restore session.
    let booted = boot(&roots, &cfg, warns.len()).await?;

    // 10.-11. playback driver (T4) + satellite subscribers (queue-persist,
    //     scrobbler, MPRIS) + control-API serve on the already-bound socket.
    //     The streaming quality is resolved from daemon_prefs through the SAME
    //     key contract the desktop uses (playback_quality(), playback.rs:
    //     170-172), so hi-res never silently downgrades. 12. QConnect (T9/T10)
    //     splices in after this, reading `booted`.
    let prefs = daemon_prefs::load_at(&roots.data);
    let quality = playback_driver::quality_from_key(&prefs.streaming_quality);
    // T11: a live-updatable cell, not a value captured once — `settings/reload`
    // re-reads `daemon_prefs` and writes here so the driver's OWN auto-advance
    // (gapless prefetch, natural-end advance) picks up a `playback.quality`
    // change without a restart. Manual play/next/prev already re-read
    // `daemon_prefs` fresh every call (api/playback.rs::resolve_quality); this
    // cell is what makes the BACKGROUND driver loop equally live.
    let quality_cell = Arc::new(std::sync::Mutex::new(quality));

    let api_audio = qbz_audio::settings::AudioSettingsStore::new_at(&roots.data)
        .map_err(|e| format!("error: could not open the audio settings store for the API: {e}"))?;
    // T11: the reload handler's "did a routing-critical field change" diff
    // needs a starting point — seed it from what's on disk right now (the same
    // settings the Player was constructed with at step 6/7).
    let initial_audio_settings = api_audio.get_settings().unwrap_or_default();

    let subs = subsystems::spawn(
        &roots,
        &cfg,
        &booted,
        bound,
        bind_addr,
        quality_cell,
        api_audio,
        initial_audio_settings,
    );

    // 13. park on SIGTERM/SIGINT. NO startup audio "hygiene": both candidate
    //     fns are verified no-ops from a fresh process and re-adding them is the
    //     documented skeptic-correction #1 trap (§8.1).
    wait_for_signal().await;

    shutdown(RunHandles {
        booted,
        shutdown_tx: subs.shutdown_tx,
        driver: subs.driver,
        queue_persist: subs.queue_persist,
        scrobbler: subs.scrobbler,
        mpris: subs.mpris,
        api: subs.api,
    })
    .await;

    Ok(0) // instance lock released on drop of `_lock`
}
