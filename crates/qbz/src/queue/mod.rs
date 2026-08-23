//! Queue sidebar controller.
//!
//! Ported from the Tauri `QueuePanel.svelte`. The core's `QueueManager`
//! owns the authoritative track list; this controller owns the
//! *sidebar-local* view state — which tab is shown, the search query, the
//! current paginator page. The NOW PLAYING heart reads the shared
//! `fav_cache` set (disk-seeded, network-refreshed) like every other
//! track surface, so it stays correct offline.
//!
//! `refresh` pulls a fresh `get_queue_state_full()` snapshot, applies the
//! active search filter, slices out the current 40-track page, and pushes
//! everything onto the `QueueState` Slint global. Every queue mutation
//! (play / remove / clear / page change / search) calls back into
//! `refresh` so the UI and the core never drift.

mod actions;
mod artwork;
mod nav;
mod paging;
mod paging_ui;
mod playlist;
mod refresh;
mod row;
mod stop_after;

#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex};

use crate::adapter::SlintAdapter;
use crate::AppWindow;

/// Upcoming tracks shown per paginator page. PAGINATED (not a growing list) to
/// keep CPU/rendering bounded on huge queues (1000+ tracks) — owner preference.
/// The cross-page drag (moving a dragged row to another page) is a SEPARATE
/// pending rework (paginator drop-zones + drag auto-scroll), tracked on its own.
pub const PAGE_SIZE: usize = 40;

type Runtime = Arc<qbz_app::shell::AppRuntime<SlintAdapter>>;

/// Sidebar-local view state. Wrapped in a `Mutex` and shared as an `Arc`
/// across every queue callback closure.
#[derive(Default)]
struct ViewState {
    /// Active tab: 0 = Queue, 1 = History.
    tab: i32,
    /// Live search query (filters the upcoming list, case-insensitive).
    search: String,
    /// Zero-based paginator page within the (filtered) upcoming list.
    page: usize,
}

/// The Queue sidebar controller — see the module docs.
pub struct QueueController {
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    playback: qbz_app::settings::playback::PlaybackPreferencesState,
    view: Arc<Mutex<ViewState>>,
    /// Last coverflow flat id-sequence fingerprint pushed to `QueueState`.
    /// `refresh_async` compares the freshly-computed hash to this: equal means a
    /// PURE ADVANCE/JUMP (same id-sequence, only the current pointer moved) — the
    /// flat model setter is SKIPPED and only `coverflow-index` is updated, so the
    /// Repeater never rebuilds and visible covers never re-decode. A different
    /// hash (new queue / shuffle / add / remove) triggers the one-time rebuild.
    /// `None` = nothing pushed yet (first refresh always rebuilds).
    last_coverflow_seq: Arc<Mutex<Option<u64>>>,
}

// `PlaybackPreferencesState` is not `Clone`, but its sole field is an
// `Arc`-shared store handle, so the controller can be cloned cheaply by
// sharing that handle. Every other field is already `Arc`/`Clone`.
impl Clone for QueueController {
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            weak: self.weak.clone(),
            handle: self.handle.clone(),
            playback: qbz_app::settings::playback::PlaybackPreferencesState {
                store: Arc::clone(&self.playback.store),
            },
            view: Arc::clone(&self.view),
            last_coverflow_seq: Arc::clone(&self.last_coverflow_seq),
        }
    }
}

impl QueueController {
    pub fn new(
        runtime: Runtime,
        weak: slint::Weak<AppWindow>,
        handle: tokio::runtime::Handle,
        playback: qbz_app::settings::playback::PlaybackPreferencesState,
    ) -> Self {
        Self {
            runtime,
            weak,
            handle,
            playback,
            view: Arc::new(Mutex::new(ViewState::default())),
            last_coverflow_seq: Arc::new(Mutex::new(None)),
        }
    }

    /// Accessors so background flows reachable only through the global
    /// controller can re-push now-playing without threading the runtime
    /// through every detail-view entry point.
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    pub fn weak(&self) -> &slint::Weak<AppWindow> {
        &self.weak
    }
    pub fn handle(&self) -> &tokio::runtime::Handle {
        &self.handle
    }
}

