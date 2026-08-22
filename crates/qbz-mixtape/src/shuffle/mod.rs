//! DJ-mix sampling for Collections / Mixtapes.
//!
//! Pure functions used by the unique-track-count and shuffle-tracks paths.
//! No Tauri types here — fully unit-testable.
//!
//! See spec: qbz-nix-docs/superpowers/specs/2026-04-25-track-shuffle-mix-design.md

mod dedup;
mod normalize;
mod sample;
mod similarity;

pub use dedup::{dedup_by_similarity, unique_track_count};
pub use normalize::{normalize_artist, normalize_title};
pub use sample::hybrid_sample;
pub use similarity::token_set_ratio;

/// Tracks whose normalized titles score at or above this Jaro/token-set
/// threshold are considered the same song (within the same normalized artist
/// bucket).
pub const SIMILARITY_THRESHOLD: f32 = 0.80;

/// No single album may contribute more than this fraction of the requested
/// sample size, after applying [`ALBUM_CAP_MIN`] as a floor.
pub const ALBUM_CAP_PCT: f32 = 0.30;

/// Floor for the per-album cap so that small requested sizes do not feel
/// artificially trimmed (e.g. requested = 20, cap = max(2, 6) = 6).
pub const ALBUM_CAP_MIN: usize = 2;

#[cfg(test)]
mod tests;
