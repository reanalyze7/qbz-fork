use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tokio::sync::broadcast;

use crate::adapter::DaemonAdapter;
use crate::paths::ProfileRoots;
use crate::state::DaemonShared;
use qbz_app::shell::AppRuntime;
use qbz_audio::settings::AudioSettingsStore;
use qbz_models::CoreEvent;

/// A socket bound at boot step 5, not yet serving. Wraps the tiny_http server
/// in an `Arc` so the serving thread and the shutdown handle can both hold it
/// (`unblock` from the handle terminates the thread's `incoming_requests`).
pub struct BoundServer {
    pub(super) server: Arc<tiny_http::Server>,
}

/// Everything the route handlers read. Owned by the single serving thread
/// (moved into it by [`serve`]), so it only needs `Send`, never `Sync` — which
/// is why a plain `AudioSettingsStore` (rusqlite `Connection`: Send, not Sync)
/// can live here directly. `token` is the opt-in `[server] token`, read once at
/// boot (`None` = open).
pub struct ApiState {
    pub runtime: Arc<AppRuntime<DaemonAdapter>>,
    pub shared: Arc<Mutex<DaemonShared>>,
    /// The CoreEvent bus (DaemonAdapter sender). `/api/events` subscribes a
    /// receiver per SSE connection; no other route touches it.
    pub bus: broadcast::Sender<CoreEvent>,
    pub roots: ProfileRoots,
    pub token: Option<String>,
    /// The bound address, echoed verbatim by `/api/info`.
    pub bind: String,
    /// Handle to the daemon's tokio runtime — the serving thread is a plain
    /// `std::thread`, so async core calls (`get_queue_state`) run via
    /// `Handle::block_on` (never called from a runtime worker → no panic).
    pub rt: tokio::runtime::Handle,
    /// Second read-only connection to the daemon-root audio settings DB (WAL
    /// allows it alongside the Player's). Supplies `configured_device`/`backend`.
    pub audio: AudioSettingsStore,
    /// Cached device enumeration for `device_present` (refreshed on a TTL so a
    /// `status` poll never re-enumerates CPAL on every call).
    pub devices: Mutex<DeviceCache>,
    /// T11: the `AudioSettings` last applied to the `Player`, so
    /// `POST /api/settings/reload` can tell whether a routing-critical field
    /// changed since the previous reload (`daemon::audio_routing_changed`) —
    /// reinit only when it did, never on every unrelated nudge.
    pub audio_snapshot: Mutex<qbz_audio::settings::AudioSettings>,
    /// T11: the live cell the playback driver's background auto-advance reads
    /// for streaming quality (`daemon.rs::run`'s `quality_cell`) — reload
    /// writes a fresh value here after re-reading `daemon_prefs`.
    pub quality: Arc<Mutex<qbz_models::Quality>>,
}

/// TTL-cached output-device names for the `device_present` check.
#[derive(Default)]
pub struct DeviceCache {
    pub at: Option<Instant>,
    pub names: Vec<String>,
}

/// Live serving handle. [`ApiHandle::shutdown`] unblocks the serving thread and
/// joins it — dropping the thread's `ApiState` (and with it the `Arc<AppRuntime>`
/// clone) BEFORE the daemon drops the runtime, preserving the §8.2 audio
/// clock-release ordering (the API thread is one more `Arc<AppRuntime>` holder,
/// exactly like the driver and auth-retry tasks).
pub struct ApiHandle {
    pub(super) server: Arc<tiny_http::Server>,
    pub(super) thread: Option<std::thread::JoinHandle<()>>,
}

impl ApiHandle {
    pub fn shutdown(mut self) {
        self.server.unblock();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Why a bind failed — `AddrInUse` is the case the boot step-5 diagnosis probes
/// (foreign qbzd vs another process); everything else is a generic fatal.
#[derive(Debug)]
pub enum BindError {
    AddrInUse(SocketAddr),
    Other(String),
}
