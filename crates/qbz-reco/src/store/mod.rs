//! SQLite-based storage for artist vectors
//!
//! Persists artist index mapping and sparse vectors for similarity search.
//! Ported 1:1 from the Tauri `artist_vectors::store`, minus the dead
//! `find_nearest` cosine path (epic D3) and the Tauri `State`/tokio wrapper —
//! the per-user lifecycle lives in the frontend/core layer (ADR-006). The
//! 3-table schema is kept byte-identical (`CREATE IF NOT EXISTS`) so the
//! `artist_vectors.db` file is reusable cross-frontend.

mod index;
mod init;
mod related;
#[cfg(test)]
mod tests;
mod vectors;

use rusqlite::Connection;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// TTL for vector entries (7 days)
pub const VECTOR_TTL_SECS: i64 = 7 * 24 * 60 * 60;

/// Artist vector store with SQLite backend
pub struct ArtistVectorStore {
    conn: Connection,
    /// In-memory cache of MBID to index mapping
    artist_to_idx: HashMap<String, u32>,
    /// Reverse mapping: index to MBID
    idx_to_artist: Vec<String>,
    /// Next available index
    next_idx: u32,
}

/// Result of a similarity search
#[derive(Debug, Clone)]
pub struct SimilarArtist {
    pub mbid: String,
    pub name: Option<String>,
    pub similarity: f32,
}

/// Get current Unix timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
