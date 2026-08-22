//! The full external-recommendations result for the Discover section.

use serde::{Deserialize, Serialize};

use super::reco::{AlbumReco, ArtistReco, TrackReco};

/// Empty vecs self-hide their row in the view, so partial population is always
/// safe — the controller paints each row independently as it resolves.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalCarousels {
    /// No connected external source -> editorial fallback regime.
    pub editorial_fallback: bool,
    /// Recommended artists from your COMMON taste (overall top -> similar, not heard).
    pub rec_artists_common: Vec<ArtistReco>,
    /// Recommended artists from your RECENT taste (1-month top -> similar, not heard).
    pub rec_artists_recent: Vec<ArtistReco>,
    /// Recommended albums (Last.fm artist top-albums, not scrobbled).
    pub rec_albums: Vec<AlbumReco>,
    /// Fresh releases (ListenBrainz, from artists you follow).
    pub fresh_releases: Vec<AlbumReco>,
    /// Weekly Exploration (ListenBrainz discovery playlist) tracks.
    pub weekly_exploration: Vec<TrackReco>,
    /// Weekly Jams (ListenBrainz familiar playlist) tracks.
    pub weekly_jams: Vec<TrackReco>,
    /// Deep-cut albums from artists you know.
    pub deep_cut_albums: Vec<AlbumReco>,
    /// Cold-start fallback: Qobuz editorial top albums.
    pub top_albums: Vec<AlbumReco>,
    /// Cold-start fallback: Qobuz editorial top artists.
    pub top_artists: Vec<ArtistReco>,
}
