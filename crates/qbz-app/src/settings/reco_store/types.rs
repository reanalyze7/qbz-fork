// ---------------------------------------------------------------------------
// Event / item types (mirror src-tauri/src/reco_store/mod.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoEventType {
    Play,
    Favorite,
    PlaylistAdd,
}

impl RecoEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Play => "play",
            Self::Favorite => "favorite",
            Self::PlaylistAdd => "playlist_add",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoItemType {
    Track,
    Album,
    Artist,
}

impl RecoItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Album => "album",
            Self::Artist => "artist",
        }
    }
}

/// A single recommendation event to persist (mirrors `RecoEventInput`).
#[derive(Debug, Clone)]
pub struct RecoEventInput {
    pub event_type: RecoEventType,
    pub item_type: RecoItemType,
    pub track_id: Option<u64>,
    pub album_id: Option<String>,
    pub artist_id: Option<u64>,
    pub playlist_id: Option<u64>,
    pub genre_id: Option<u64>,
}

/// A top-artist seed (mirrors `TopArtistSeed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopArtistSeed {
    pub artist_id: u64,
    pub play_count: u32,
}

/// The ID seeds for the home/Discover recommendation rows (mirrors `HomeSeeds`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HomeSeeds {
    pub recently_played_album_ids: Vec<String>,
    pub continue_listening_track_ids: Vec<u64>,
    pub top_artist_ids: Vec<TopArtistSeed>,
    pub favorite_album_ids: Vec<String>,
    pub favorite_track_ids: Vec<u64>,
}

/// Limits for a `get_home_seeds` call (mirrors the four `v2_reco_get_home*` args).
#[derive(Debug, Clone, Copy)]
pub struct HomeSeedLimits {
    pub recent_albums: u32,
    pub continue_tracks: u32,
    pub top_artists: u32,
    pub favorites: u32,
}

impl Default for HomeSeedLimits {
    fn default() -> Self {
        // Same defaults as Tauri's v2_reco_get_home commands.
        Self {
            recent_albums: 12,
            continue_tracks: 10,
            top_artists: 10,
            favorites: 12,
        }
    }
}

/// Parameters for `train()` (mirrors `v2_reco_train_scores` args + defaults).
#[derive(Debug, Clone, Copy)]
pub struct TrainParams {
    pub lookback_days: i64,
    pub half_life_days: f64,
    pub max_events: u32,
    pub max_per_type: u32,
}

impl Default for TrainParams {
    fn default() -> Self {
        Self {
            lookback_days: 90,
            half_life_days: 21.0,
            max_events: 5000,
            max_per_type: 200,
        }
    }
}
