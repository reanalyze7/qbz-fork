//! Location-based artist discovery using MusicBrainz
//!
//! Implements the scene discovery pipeline:
//! 1. Extract artist metadata (location, genres) from MusicBrainz
//! 2. Browse candidates by area with genre affinity scoring
//! 3. Validate candidates against Qobuz catalog

mod affinity;
mod country_codes;
mod dates;
mod resolve;

pub use affinity::{build_scene_cache_key, compute_affinity_score};
pub use dates::format_life_span_date;
pub use resolve::extract_metadata;
