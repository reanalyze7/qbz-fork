//! Human-readable suggestion-reason generation (dev mode).

use super::SuggestionsEngine;

impl SuggestionsEngine {
    /// Generate a human-readable reason for suggestion
    pub(super) fn generate_reason(
        &self,
        _artist_mbid: &str,
        artist_name: Option<&str>,
        similarity: f32,
        _playlist_artists: &[String],
    ) -> String {
        let name = artist_name.unwrap_or("Unknown");
        let score_pct = (similarity * 100.0).round() as u32;

        format!("Similar to your playlist ({score_pct}% match) - {name}")
    }
}
