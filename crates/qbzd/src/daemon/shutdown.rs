use qbz_app::playback_driver;
use tokio::task::JoinHandle;

use crate::mpris::MprisHandle;

use super::BootedRuntime;

/// Everything [`super::run::run`] hands off to [`shutdown`] once it returns
/// from the signal park. Bundled into one struct purely so the call site
/// reads as one statement — the ordering below is unchanged from the
/// original inline sequence (§8.2) and must NOT be reshuffled.
pub(super) struct RunHandles {
    pub(super) booted: BootedRuntime,
    pub(super) shutdown_tx: tokio::sync::watch::Sender<bool>,
    pub(super) driver: JoinHandle<()>,
    pub(super) queue_persist: JoinHandle<()>,
    pub(super) scrobbler: JoinHandle<()>,
    pub(super) mpris: Option<MprisHandle>,
    pub(super) api: crate::api::ApiHandle,
}

/// The ordered shutdown (§8.2). Do NOT extract any piece of this sequence
/// into a further function that could get reordered by a future edit — the
/// `Arc<AppRuntime>` refcount / #521 clock-release invariant depends on this
/// exact order, kept inline exactly as it ran inside `run()` before the
/// split.
pub(super) async fn shutdown(mut h: RunHandles) {
    // Step 1: stop the playback driver. It holds an Arc<AppRuntime> clone, so its
    // task must finish (dropping that Arc) before `drop(booted)` can release
    // the audio device ahead of the #521 pair. Signal, then join.
    let _ = h.shutdown_tx.send(true);
    if let Err(e) = h.driver.await {
        log::warn!("driver task join failed: {e:?}");
    }
    // T10 (§7.5): stop the queue-persistence subscriber before the authoritative
    // final save, so it neither races the flush below nor keeps its
    // `Arc<AppRuntime>` clone alive past `drop(booted)` (#521 ordering).
    h.queue_persist.abort();
    let _ = h.queue_persist.await;
    // Stop the scrobble-on-play subscriber (holds no Arc<AppRuntime>; order-free).
    h.scrobbler.abort();
    let _ = h.scrobbler.await;
    // Tear down MPRIS: abort its updater and drop the D-Bus handle. Its inbound
    // callback held only a Weak<AppRuntime>, so this is order-free too.
    if let Some(mpris) = h.mpris {
        mpris.shutdown().await;
    }
    // Final full session save (queue + position) now that playback is quiesced.
    playback_driver::save_session_now(h.booted.runtime.as_ref()).await;
    // The background auth-retry task also holds an Arc<AppRuntime> clone — abort
    // AND join it so its Arc is dropped before `drop(booted)`; otherwise the
    // ordering claim below (drop releases the device) breaks once playback has
    // engaged a real device.
    if let Some(retry) = h.booted.auth_retry.take() {
        retry.abort();
        let _ = retry.await;
    }
    // Stop the API thread and JOIN it: its `ApiState` holds an `Arc<AppRuntime>`
    // clone, which must drop before `drop(booted)` releases the audio device
    // ahead of the #521 pair — the same ordering constraint as the driver and
    // auth-retry tasks (§8.2).
    h.api.shutdown();
    // Release the audio device by dropping the runtime (its Player) BEFORE the
    // #521 pair (§8.2 step 3 precedes step 4).
    drop(h.booted);
    //    THE #521 PAIR runs unconditionally on Linux — exactly the desktop quit
    //    choke-point (crates/qbz/src/main.rs:20393): a forced PipeWire clock left
    //    set would pin the whole system's sample rate after the process dies.
    //    Both calls self-gate to no-ops when QBZ forced nothing.
    #[cfg(target_os = "linux")]
    {
        qbz_audio::alsa_backend::resume_suspended_sink();
        qbz_audio::pipewire_backend::PipeWireBackend::reset_pipewire_clock();
    }
}
