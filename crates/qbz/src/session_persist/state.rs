//! Process-global store handle + cached gates + the pending-resume slot.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use qbz_app::session_store::SessionStore;
use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;

pub(super) type Runtime = Arc<AppRuntime<SlintAdapter>>;

/// Per-user session store, bound at activation (None before login / on failure).
pub(super) static STORE: Mutex<Option<SessionStore>> = Mutex::new(None);
/// Cached `persist_session` gate — captured/restored only when true.
pub(super) static PERSIST_SESSION: AtomicBool = AtomicBool::new(false);
/// Cached `resume_playback_position` gate — primes [`PENDING_RESUME`] at restore.
pub(super) static RESUME_POSITION: AtomicBool = AtomicBool::new(false);
/// Position (secs) to resume the restored current track at on the first play.
/// 0 = none. Written at restore (when resume is on); consumed on first play.
pub(super) static PENDING_RESUME: AtomicU64 = AtomicU64::new(0);
/// Track id the pending resume position applies to, so ONLY the restored current
/// track resumes — playing any other track first starts from 0. 0 = none.
pub(super) static PENDING_RESUME_TRACK: AtomicU64 = AtomicU64::new(0);
/// Runtime + tokio handle captured at shell entry, so the synchronous window
/// close handlers can flush a final full snapshot before the loop quits.
pub(super) static EXIT_CTX: OnceLock<(Runtime, tokio::runtime::Handle)> = OnceLock::new();

/// Bind the runtime + tokio handle for `save_on_exit`. Called once at shell
/// entry (idempotent — later calls are ignored by `OnceLock`).
pub fn bind_exit_ctx(runtime: Runtime, handle: tokio::runtime::Handle) {
    let _ = EXIT_CTX.set((runtime, handle));
}

/// Refresh the cached gate flags (called by the Settings toggle handlers and the
/// settings snapshot load so the cache tracks live preference changes).
pub fn set_gates(persist_session: bool, resume_position: bool) {
    PERSIST_SESSION.store(persist_session, Ordering::Relaxed);
    RESUME_POSITION.store(resume_position, Ordering::Relaxed);
}

/// Whether session persistence is currently enabled.
pub fn persist_enabled() -> bool {
    PERSIST_SESSION.load(Ordering::Relaxed)
}

/// Take + clear the pending resume position IF it applies to `track_id` (the
/// restored current track). Returns the saved position once, then 0 forever —
/// and 0 for any other track, so playing something else first never resumes.
pub fn take_resume_for(track_id: u64) -> u64 {
    if track_id != 0 && PENDING_RESUME_TRACK.swap(0, Ordering::Relaxed) == track_id {
        PENDING_RESUME.swap(0, Ordering::Relaxed)
    } else {
        0
    }
}

/// Peek the pending resume position WITHOUT consuming it (so the seek bar can be
/// seeded at restore while the actual resume still fires on first play). 0 = none.
pub fn pending_resume_position() -> u64 {
    if PENDING_RESUME_TRACK.load(Ordering::Relaxed) != 0 {
        PENDING_RESUME.load(Ordering::Relaxed)
    } else {
        0
    }
}
