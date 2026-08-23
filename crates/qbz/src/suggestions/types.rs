//! Payload types shared across the suggestions pipeline.

/// A resolved playlist card (book collage of up to 3 distinct album covers).
pub(crate) struct PlaylistCard {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) track_count: u32,
    /// Up to 3 distinct album-cover URLs for the book collage.
    pub(super) cover_urls: Vec<String>,
}

/// The fully-assembled suggestions for one (artist, track) pair.
pub struct SuggestionsPayload {
    pub artist_id: String,
    pub seed_track_id: String,
    pub seed_track_name: String,
    pub seed_artist_id: String,
    pub playlist_cards: Vec<PlaylistCard>,
    pub rec_tracks: Vec<qbz_models::Track>,
    /// Up to 4 distinct rec-track album covers for the radio diamond collage.
    pub radio_cover_urls: Vec<String>,
    pub error: bool,
}

/// An empty payload (no cards, no tracks, no error) — the "no track selected"
/// reset state applied when the immersive panel opens with no current track.
pub fn empty_payload() -> SuggestionsPayload {
    SuggestionsPayload {
        artist_id: String::new(),
        seed_track_id: String::new(),
        seed_track_name: String::new(),
        seed_artist_id: String::new(),
        playlist_cards: Vec::new(),
        rec_tracks: Vec::new(),
        radio_cover_urls: Vec::new(),
        error: false,
    }
}
