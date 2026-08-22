//! Genre and tag normalization for MusicBrainz scene discovery
//!
//! Separates genres (primary signals) from tags (secondary signals),
//! filters noise, and normalizes equivalent names.

mod extract;
mod normalize;
mod tables;
#[cfg(test)]
mod tests;

pub use extract::{extract_affinity_seeds, genre_summary};
pub use normalize::{is_broad_genre, normalize_genre};
