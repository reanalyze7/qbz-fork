use std::sync::Arc;

use qbz_app::playback_driver;
use tokio::task::JoinHandle;

use crate::config::QbzdConfig;
use crate::mpris::MprisHandle;
use crate::paths::ProfileRoots;

use super::driver_deps::build_driver_deps;
use super::queue_persist::spawn_queue_persist;
use super::BootedRuntime;

/// Steps 10-11 of §8.1: spawn the playback driver plus its satellite
/// subscribers (queue-persist, scrobbler, MPRIS), then serve the control API
/// on the already-bound socket. Bundled into one function so `run()` stays
/// readable; the returned handles are threaded straight into the ordered
/// shutdown sequence unchanged.
pub(super) struct Subsystems {
    pub(super) shutdown_tx: tokio::sync::watch::Sender<bool>,
    pub(super) driver: JoinHandle<()>,
    pub(super) queue_persist: JoinHandle<()>,
    pub(super) scrobbler: JoinHandle<()>,
    pub(super) mpris: Option<MprisHandle>,
    pub(super) api: crate::api::ApiHandle,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn(
    roots: &ProfileRoots,
    cfg: &QbzdConfig,
    booted: &BootedRuntime,
    bound: crate::api::BoundServer,
    bind_addr: std::net::SocketAddr,
    quality_cell: Arc<std::sync::Mutex<qbz_models::Quality>>,
    api_audio: qbz_audio::settings::AudioSettingsStore,
    initial_audio_settings: qbz_audio::settings::AudioSettings,
) -> Subsystems {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    // T10 (§7.2): the driver's `ReportEdge` action pulses this Notify; the
    // QConnect report scheduler (step 12) waits on it. Created BEFORE the driver
    // so `on_edge` can capture it, and shared with `qconnect::start`.
    let report_notify = Arc::new(tokio::sync::Notify::new());
    let deps = build_driver_deps(quality_cell.clone(), booted.shared.clone(), report_notify);
    let driver = tokio::spawn(playback_driver::run_driver(
        booted.runtime.clone(),
        deps,
        shutdown_rx,
    ));

    // 10b. Queue-persistence subscriber (T10, §7.5): a `CoreEvent::QueueUpdated`
    //      on the DaemonAdapter bus — from driver auto-advance, the CLI queue
    //      verbs OR a QConnect-driven remote mutation — is debounced 2 s and then
    //      flushed to the session store, so a restart resumes the remote-set queue
    //      PAUSED (boot already restores it, §8.1-9½). Holds an `Arc<AppRuntime>`
    //      clone, so it is aborted+joined ahead of `drop(booted)` (#521 ordering).
    let queue_persist = spawn_queue_persist(booted.runtime.clone(), booted.bus.subscribe());

    // 10c. Scrobble-on-play (CONSOLE): a CoreEvent-bus subscriber that sends
    //      "now playing" on TrackStarted and scrobbles once past the Last.fm
    //      threshold, to whichever of Last.fm / ListenBrainz is connected +
    //      enabled in the scrobbler store. Holds NO Arc<AppRuntime>, so it sits
    //      outside the #521/§8.2 ordering — aborted for a clean shutdown below.
    let scrobbler = crate::scrobble_engine::spawn(roots.clone(), booted.bus.subscribe());

    // 10d. MPRIS media controls (CONSOLE): publish org.mpris.MediaPlayer2 so a
    //      KDE/GNOME media widget, a plasmoid, or hardware media keys drive the
    //      daemon with no custom client. The inbound callback holds a
    //      Weak<AppRuntime> (never pins the runtime), so it too sits outside the
    //      #521 ordering; None on a headless box / when QBZD_MPRIS disables it.
    let mpris = crate::mpris::spawn(
        &booted.runtime,
        roots.clone(),
        booted.bus.subscribe(),
        tokio::runtime::Handle::current(),
    );

    // 11. HTTP serve (02 §3) on the already-bound socket. `ApiState` carries a
    //     second read-only audio-store connection (WAL) for the status audio
    //     block, the tokio handle for the async queue read, and the opt-in
    //     [server] token (None = open). 12. QConnect (T9/T10) splices after this.
    let api = crate::api::serve(
        bound,
        crate::api::ApiState {
            runtime: booted.runtime.clone(),
            shared: booted.shared.clone(),
            bus: booted.bus.clone(),
            roots: roots.clone(),
            token: cfg.server.token.clone().filter(|t| !t.trim().is_empty()),
            bind: bind_addr.to_string(),
            rt: tokio::runtime::Handle::current(),
            audio: api_audio,
            devices: std::sync::Mutex::new(crate::api::DeviceCache::default()),
            audio_snapshot: std::sync::Mutex::new(initial_audio_settings),
            quality: quality_cell,
        },
    );
    log::info!("control API listening on {bind_addr}");

    Subsystems {
        shutdown_tx,
        driver,
        queue_persist,
        scrobbler,
        mpris,
        api,
    }
}
