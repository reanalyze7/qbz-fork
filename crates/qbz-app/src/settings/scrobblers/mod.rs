//! Portable scrobbler credential storage (Last.fm + ListenBrainz).
//!
//! Owns the persisted scrobbler settings for the Slint frontend, the part the
//! Tauri build kept in per-user localStorage:
//!   - the master enable + collapse UI flags,
//!   - Last.fm: session key + username + per-service enable flag
//!     (replaces the `qbz-lastfm-session-key` / `qbz-lastfm-scrobbling`
//!     localStorage keys — webview localStorage is unreachable from Slint,
//!     so Last.fm credentials cannot be shared with the Tauri build),
//!   - ListenBrainz: token + username + per-service enable flag. The token is
//!     ALSO written through to the shared `ListenBrainzCache.credentials` row
//!     by the Slint `scrobble` controller, so LB credentials DO stay shared
//!     with the Tauri build; this copy is the Slint UI's fast read.
//!
//! The Last.fm offline queue does NOT live here: it is the `scrobble_queue`
//! table in the shared per-user `offline_settings.db`
//! ([`crate::offline_mode::store::OfflineModeStore`]) — same rows Tauri queues
//! into and flushes from. The ListenBrainz offline queue is the
//! `ListenBrainzCache.listen_queue` (qbz-integrations).
//!
//! Mirrors the other per-user settings stores: a single-row SQLite settings
//! table re-pointed at the active user's data directory via
//! [`ScrobblerSettingsState::init_at`] at login, so credentials are scoped per
//! Qobuz user.
//!
//! Runtime concerns — the auth flows, the now-playing + scrobble fire, the
//! `min(50% of duration, 240s)` timer, the offline flush — live in the Slint
//! `scrobble` controller and call the `qbz-integrations` clients directly.

mod lastfm_ops;
mod listenbrainz_ops;
mod state;
mod store;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use state::ScrobblerSettingsState;
pub use store::ScrobblerSettingsStore;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScrobblerSettings {
    /// Master toggle. When false, the whole scrobblers section body is hidden.
    /// Default OFF — integrations are opt-in.
    pub enabled: bool,
    /// Collapse chevron state. Body renders only when `enabled && !ui_collapsed`.
    pub ui_collapsed: bool,

    // --- Last.fm ---
    /// Per-service scrobbling enable flag (replaces `qbz-lastfm-scrobbling`).
    pub lastfm_enabled: bool,
    /// Last.fm session key (`get_session().key`). Empty = not authenticated.
    pub lastfm_session_key: String,
    /// Last.fm username (`LastFmSession.name`), for the "Signed in as …" label.
    pub lastfm_username: String,

    // --- ListenBrainz ---
    /// Per-service enable flag.
    pub listenbrainz_enabled: bool,
    /// ListenBrainz user token. Empty = not authenticated.
    pub listenbrainz_token: String,
    /// ListenBrainz username (`UserInfo.user_name`).
    pub listenbrainz_username: String,
}

impl ScrobblerSettings {
    /// Last.fm has credentials (independent of the enable flags).
    pub fn lastfm_is_authed(&self) -> bool {
        !self.lastfm_session_key.is_empty()
    }

    /// ListenBrainz has credentials (independent of the enable flags).
    pub fn listenbrainz_is_authed(&self) -> bool {
        !self.listenbrainz_token.is_empty()
    }

    /// Last.fm should actually scrobble: master + service on + authed.
    pub fn lastfm_active(&self) -> bool {
        self.enabled && self.lastfm_enabled && self.lastfm_is_authed()
    }

    /// ListenBrainz should actually scrobble: master + service on + authed.
    pub fn listenbrainz_active(&self) -> bool {
        self.enabled && self.listenbrainz_enabled && self.listenbrainz_is_authed()
    }
}
