//! Per-user offline-mode settings store.
//!
//! Opens the SAME `offline_settings.db` Tauri's `src-tauri/src/offline/mod.rs`
//! uses (identical creation SQL + additive migrations), so the file stays
//! frontend-portable like `library.db`/`index.db`. This shared store exposes
//! only the subset the offline-MODE port consumes:
//!
//! - `manual_offline_mode` — the induced-offline flag (persisted; D1).
//! - `show_network_folders_in_manual_offline` — network-mount policy (D9).
//! - `pre_offline_stream_first_track` — the issue #279 snapshot of
//!   `audio_settings.stream_first_track` taken on entering induced offline.
//!
//! The legacy columns/tables (cast/scrobbling flags, `pending_playlist_sync`,
//! `scrobble_queue`, `cache_limit_bytes`) are still CREATED for byte-level
//! compatibility with the Tauri schema, but get no API here: the dead toggles
//! are not ported (spec §1) and offline playlist creation is replaced by
//! first-class local playlists (D7/D8).

mod schema;
mod scrobble_queue;
mod settings;
#[cfg(test)]
mod tests;
mod types;

pub use types::{OfflineModeSettings, QueuedScrobble};

use rusqlite::Connection;

pub struct OfflineModeStore {
    conn: Connection,
}
