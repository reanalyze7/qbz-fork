//! Frontend-agnostic external-recommendations engine for Discover (ADR-006).
//!
//! Blends Last.fm + ListenBrainz into the Discover "Recommendations" tab,
//! validating every candidate against the Qobuz catalog before display (the app
//! can only play Qobuz content). Lineup (owner-directed 2026-06-27):
//!   - Recommended Artists (Last.fm similar of your recent top, not heard).
//!   - Recommended Albums (Last.fm artist top-albums, not scrobbled).
//!   - Fresh Releases (ListenBrainz, from artists you follow).
//!   - Weekly Exploration / Weekly Jams (ListenBrainz curated playlists).
//!   - Deep-cut albums from artists you know.
//!   - Cold-start fallback: Qobuz editorial top albums + artists.
//!
//! The per-row builders are public so the frontend can paint each row the moment
//! it resolves (progressive load). The "heard" filters compare against the LOCAL
//! reco-store history + a light Last.fm/LB top-set (not a full history sweep).

pub mod cache;
pub mod matching;
pub mod types;

mod carousels;
mod combine;
mod validate;

use std::sync::Mutex;

use qbz_integrations::{LastFmClient, ListenBrainzClient, MusicBrainzClient};
use qbz_models::{Album, Artist, Track};

pub use cache::RecoCache;
pub use carousels::{compose_artist_rails, ArtistRailComposition, ARTIST_DISPLAY_CAP};
pub use combine::{
    build_deep_cut_albums, build_editorial, build_external_carousels, build_fresh_releases,
    build_rec_albums, build_rec_artists_common, build_rec_artists_recent,
    build_similar_albums_seeded, build_weekly_exploration, build_weekly_jams,
};
pub use types::{
    AlbumReco, ArtistReco, ExternalCarousels, ExtHistory, LocalHistory, RecoSource, TrackReco,
};

/// The Qobuz catalog operations the engine needs. Implemented by the frontend
/// over its own `QbzCore`. Every method swallows errors to an empty result.
#[async_trait::async_trait]
pub trait RecoCatalog: Send + Sync {
    async fn search_tracks(&self, query: &str, limit: usize) -> Vec<Track>;
    async fn search_artists(&self, query: &str, limit: usize) -> Vec<Artist>;
    /// Free-text Qobuz album search (for validating a recommended album).
    async fn search_albums(&self, query: &str, limit: usize) -> Vec<Album>;
    async fn artist_top_tracks(&self, artist_id: u64, limit: usize) -> Vec<Track>;
    /// An artist's albums (the deep-cut candidate source).
    async fn artist_albums(&self, artist_id: u64, limit: usize) -> Vec<Album>;
    /// Editorial featured albums by kind ("most-streamed" | "new-releases" | …).
    async fn featured_albums(&self, kind: &str, limit: usize) -> Vec<Album>;
    async fn get_artist(&self, artist_id: u64) -> Option<Artist>;
}

pub struct LastFmHandle<'a> {
    pub username: String,
    pub client: &'a LastFmClient,
}

pub struct ListenBrainzHandle<'a> {
    pub username: String,
    pub client: &'a ListenBrainzClient,
}

pub struct RecoInputs<'a> {
    pub lastfm: Option<LastFmHandle<'a>>,
    pub listenbrainz: Option<ListenBrainzHandle<'a>>,
    pub musicbrainz: &'a MusicBrainzClient,
    pub catalog: &'a dyn RecoCatalog,
    pub cache: Option<&'a Mutex<RecoCache>>,
    pub local: LocalHistory,
    /// Daily rotation offset (e.g. days since the Unix epoch).
    pub rotation_seed: u64,
}

impl RecoInputs<'_> {
    /// Whether any external source is connected.
    pub fn has_external(&self) -> bool {
        self.lastfm.is_some() || self.listenbrainz.is_some()
    }
}

/// True when no external source is connected -> editorial fallback regime.
pub fn is_cold_start(inputs: &RecoInputs<'_>) -> bool {
    !inputs.has_external()
}

/// Gather the external "heard" history ONCE (shared across all row builders).
pub async fn gather_history(inputs: &RecoInputs<'_>) -> ExtHistory {
    carousels::gather_history(inputs).await
}
