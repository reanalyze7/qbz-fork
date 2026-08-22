//! Match imported tracks to Qobuz catalog

mod matcher;
mod normalize;
mod scoring;

pub use matcher::match_tracks;
