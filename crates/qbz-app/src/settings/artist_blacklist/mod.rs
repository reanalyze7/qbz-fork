//! Headless artist blacklist service.
//!
//! Frontend-agnostic 1:1 port of the Tauri `BlacklistService`
//! (`src-tauri/src/artist_blacklist/service.rs` + `models.rs`). No
//! `tauri::State`, per ADR-006 and the V2 "move logic to a core crate, never
//! wrap legacy" rule. The DB filename, schema, and pragmas are kept IDENTICAL
//! to the Tauri store so existing users' `artist_blacklist.db` keeps working.
//!
//! Provides O(1) artist blacklist checks via an in-memory `HashSet` backed by
//! SQLite persistence, plus a global enable/disable feature flag.

mod albums;
mod albums_admin;
mod artists;
mod flags;
mod lifecycle;
mod schema;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

use rusqlite::Connection;

/// Database file name for the artist blacklist store.
///
/// Kept identical to the Tauri store so the later lifecycle layer opens the
/// same per-user database.
pub const DB_FILE_NAME: &str = "artist_blacklist.db";

/// A blacklisted artist entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistedArtist {
    pub artist_id: u64,
    pub artist_name: String,
    pub added_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// A blacklisted album entry.
///
/// The album axis is a parallel, `String`-keyed pipeline alongside the
/// `u64` artist one: Qobuz album ids are alphanumeric strings, so they
/// cannot be stored in the artist table's INTEGER primary key. This is its
/// own table in the same database. Blocking an album hides it by its OWN
/// id regardless of artist — the surgical fix for Qobuz's same-name artist
/// merges (e.g. a Trance "Anthrax" release landing on the Thrash Anthrax id).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistedAlbum {
    pub album_id: String,
    pub album_title: String,
    pub artist_name: String,
    pub cover_url: String,
    pub added_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Blacklist settings (enable/disable toggle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistSettings {
    pub enabled: bool,
}

impl Default for BlacklistSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Artist blacklist service with O(1) lookup performance.
pub struct BlacklistService {
    pub(super) conn: Connection,
    /// In-memory set for O(1) lookups.
    pub(super) blacklisted_ids: RwLock<HashSet<u64>>,
    /// In-memory set of blocked album ids (String-keyed) for O(1) lookups.
    pub(super) blacklisted_album_ids: RwLock<HashSet<String>>,
    /// Feature flag - when false, `is_blacklisted()` always returns false.
    /// Shared by both axes: it also gates `is_album_blacklisted()`.
    pub(super) enabled: AtomicBool,
}
