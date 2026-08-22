//! SQLite database layer for library persistence.
//!
//! Split from a single 6300+ line `database.rs` into one directory module
//! per domain (folders, tracks, albums, search, playlists, ...). Every
//! domain file is an `impl LibraryDatabase { ... }` block — Rust allows an
//! inherent impl to be spread across many files in the same crate, so no
//! method changed shape or name during the split. This file is the public
//! surface: the `LibraryDatabase` struct itself, `open`/`with_connection*`,
//! module wiring, and re-exports of every type used across submodules.

use rusqlite::Connection;
use std::path::Path;

use crate::LibraryError;

mod album_settings;
mod albums;
mod artist_images;
mod artists;
mod custom_album_covers;
mod custom_order;
mod folder_tree;
mod folders;
mod helpers;
mod kv;
mod local_content;
mod playlist_folders;
mod playlist_local_tracks;
mod playlist_settings;
mod playlist_stats;
mod purchases;
mod qobuz_downloads;
mod schema;
mod search;
mod sidecar_position;
mod stats;
mod tracks;
mod types;

pub use types::{
    AlbumTrackUpdate, LibraryFolder, LibraryStats, LocalContentStatus, PlaylistFolder,
    PlaylistSettings, PlaylistStats, TrackMetadataUpdateFull,
};

/// Library database wrapper
pub struct LibraryDatabase {
    conn: Connection,
}

impl LibraryDatabase {
    /// Open or create database at path
    pub fn open(db_path: &Path) -> Result<Self, LibraryError> {
        log::info!("Opening library database at: {}", db_path.display());

        let conn = Connection::open(db_path)
            .map_err(|e| LibraryError::Database(format!("Failed to open database: {}", e)))?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| LibraryError::Database(format!("Failed to set WAL mode: {}", e)))?;

        let db = Self { conn };
        db.init_schema()?;
        db.run_migrations()?;
        // First-class LOCAL playlists (offline-mode D7) — separate module,
        // same database file. Idempotent CREATE IF NOT EXISTS.
        crate::local_playlists::init_schema(&db.conn)
            .map_err(|e| LibraryError::Database(format!("local_playlists schema: {}", e)))?;
        // Qobuz playlist snapshot (offline-mode B7/B8) — names + membership
        // captured opportunistically while online. Idempotent.
        crate::qobuz_playlist_snapshot::init_schema(&db.conn)
            .map_err(|e| LibraryError::Database(format!("qobuz_playlist_snapshot schema: {}", e)))?;
        Ok(db)
    }

    /// Provide raw connection access for external schema migrations.
    ///
    /// This is intentionally narrow: callers receive a shared reference so
    /// they can run DDL (CREATE TABLE, ALTER TABLE) but cannot move the
    /// connection out or replace it.  Use sparingly — prefer adding methods
    /// to LibraryDatabase directly for DML queries.
    pub fn with_connection<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Connection) -> R,
    {
        f(&self.conn)
    }

    /// Provide mutable raw connection access for operations that require a
    /// transaction (e.g. reorder operations that delete + reinsert rows).
    ///
    /// Use sparingly — prefer adding methods to LibraryDatabase directly.
    pub fn with_connection_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Connection) -> R,
    {
        f(&mut self.conn)
    }
}
