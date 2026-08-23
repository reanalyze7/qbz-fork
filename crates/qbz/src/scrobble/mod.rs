//! Settings > Integrations — scrobbler (Last.fm + ListenBrainz) auth controller
//! AND the source-agnostic now-playing / scrobble fire.
//!
//! Source-agnostic by construction: the fire path reads the CURRENT
//! `qbz_models::QueueTrack`'s already-normalized `artist` / `album` / title
//! (Qobuz and local all funnel through it) and feeds plain text — no
//! artwork (Last.fm has no image field; ListenBrainz takes only optional MB
//! IDs / ISRC / duration). The clients live in `qbz-integrations` and are
//! called directly (no Tauri command seam).
//!
//! Two firing edges (mirrors the Svelte `playbackService.ts`):
//!   - now-playing: fires immediately on a track-change edge (skipped offline).
//!   - scrobble: armed at `min(50% of duration, 240s)` after the change; a
//!     monotonic `SCROBBLE_GEN` guard self-cancels a stale timer if the track
//!     changed before it fires (the Svelte `clearTimeout` equivalent). Like
//!     Tauri, pause does NOT stop the clock.
//!
//! Offline behavior: engine offline OR call failure queues the scrobble —
//! Last.fm into the SHARED per-user `offline_settings.db` `scrobble_queue`
//! (same rows Tauri queues/flushes), ListenBrainz into the SHARED per-user
//! `listenbrainz_v2.db` `listen_queue`. A watcher on the offline-mode engine
//! drains both queues on every offline -> online edge (manual-flag exits
//! included), plus once at shell entry.
//!
//! Persistence lives in `crate::scrobbler_settings` (the per-user
//! `scrobbler_settings.db`); the auth flows seed/clear it. ListenBrainz
//! credentials are ALSO written through to the shared `ListenBrainzCache`
//! credentials row, so the Tauri build sees the same sign-in (and a Tauri
//! sign-in seeds this build at shell entry).

use std::path::PathBuf;
use std::sync::Mutex;

use slint::{ComponentHandle, Weak};

use crate::scrobbler_settings;
use crate::{AppWindow, ScrobbleState};

mod fire;
mod fire_send;
mod flush;
mod flush_listenbrainz;
mod lastfm_auth;
mod lastfm_confirm;
mod listenbrainz_auth;
mod panel;
mod queue;
mod startup;

pub use fire::{on_track_changed, ScrobbleMeta};
pub use lastfm_auth::{lastfm_connect, lastfm_enable_toggle, lastfm_open_auth_url};
pub use lastfm_confirm::{lastfm_confirm, lastfm_disconnect};
pub use listenbrainz_auth::{
    listenbrainz_disconnect, listenbrainz_enable_toggle, listenbrainz_set_token,
};
pub use panel::{collapse_toggle, enable_toggle, load};
pub use startup::start;

// ----------------------------------------------------------------------------
// Status helper — Slint uses inline @tr, so we resolve to a plain English
// label Rust-side. `kind`: 0 none, 1 info, 2 connected/ok, 3 error.
// ----------------------------------------------------------------------------

fn set_status(weak: &Weak<AppWindow>, text: String, kind: i32) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<ScrobbleState>();
        s.set_status_text(text.into());
        s.set_status_kind(kind);
    });
}

// ----------------------------------------------------------------------------
// Last.fm pending-token bridge. `get_token` returns a request token the user
// authorizes in the browser; `get_session` (the confirm step) exchanges it.
// The two steps are separate UI callbacks, so the token is stashed here.
// ----------------------------------------------------------------------------

static LASTFM_PENDING_TOKEN: Mutex<Option<String>> = Mutex::new(None);

// ----------------------------------------------------------------------------
// Tokio runtime handle — captured at shell entry so the fire path (which runs
// from the playback poll, not a UI callback) can spawn network tasks.
// ----------------------------------------------------------------------------

static RT_HANDLE: Mutex<Option<tokio::runtime::Handle>> = Mutex::new(None);

fn rt_handle() -> Option<tokio::runtime::Handle> {
    RT_HANDLE.lock().ok().and_then(|g| g.clone())
}

/// `<user_dir>/cache/listenbrainz_v2.db` — the SAME per-user file Tauri's
/// `ListenBrainzV2State::init_cache_at` opens, so credentials and the offline
/// listen queue are shared across frontends.
fn listenbrainz_cache_path() -> Option<PathBuf> {
    let dir = scrobbler_settings::user_dir()?.join("cache");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("listenbrainz_v2.db"))
}
