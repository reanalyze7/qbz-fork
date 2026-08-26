//! Loudness normalization support
//!
//! Extracts ReplayGain metadata from audio files and calculates gain factors
//! for volume normalization. When normalization is disabled, this module is
//! not invoked and the audio pipeline remains bit-perfect.

mod extract;
pub mod gain;
mod source_adapter;
mod tag_parse;
mod tags;

pub use extract::{extract_replaygain, extract_replaygain_from_reader};
pub use gain::{
    calculate_gain_factor, db_to_linear, gain_db_for, gain_factor_for, is_plausible_lufs,
    MAX_GAIN_DB, MIN_GAIN_DB,
};

/// Extracted loudness data for a track
#[derive(Debug, Clone)]
pub struct ReplayGainData {
    /// Gain adjustment in dB (negative = reduce volume, positive = increase)
    pub gain_db: f32,
    /// Peak sample value (0.0-1.0+), used for clipping prevention
    pub peak: Option<f32>,
}
