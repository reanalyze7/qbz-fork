//! Plain data types produced by the Home / Discover worker-thread mappers
//! and consumed by the Slint-conversion layer.

use qbz_app::settings::discover_prefs::DiscoverySectionId;

/// Plain, `Send` home data produced on the worker thread.
pub struct HomeData {
    pub sections: Vec<SectionData>,
    /// Editorial-only section set for the Editor's Picks tab.
    pub editor_sections: Vec<SectionData>,
    pub popular: Vec<SlimData>,
    pub recent: Vec<SlimData>,
    pub recent_albums: Vec<CardData>,
    /// Qobuz playlists row — home tab.
    pub playlists: Vec<PlaylistCardData>,
    /// Qobuz playlists row — editorPicks tab (same data, separate cache slot).
    pub editor_playlists: Vec<PlaylistCardData>,
    /// Category tags for the Qobuz Playlists multi-select filter: (slug,
    /// localized name). Empty when the index carries no `playlists_tags`.
    pub playlist_tags: Vec<(String, String)>,
    /// "Library Albums" rail (#566) — the user's favorite albums from the
    /// SAME pipeline For You uses (`foryou::favorite_album_cards`), fetched
    /// concurrently with the discover index. Feeds
    /// `HomeState.favorite-albums`; the view arm self-hides while empty.
    pub favorite_albums: Vec<crate::foryou::AlbumCard>,
    /// "Release Watch" rail (#566) — same pipeline as For You
    /// (`foryou::fetch_release_watch`, blacklist-filtered), fetched
    /// concurrently. Feeds `HomeState.release-watch`; self-hides while empty.
    pub release_watch: Vec<crate::foryou::AlbumCard>,
    /// "Your Top Artists" rail (#566) — same pipeline as For You
    /// (`foryou::top_artist_cards`), fetched concurrently. Feeds
    /// `HomeState.top-artists`; self-hides while empty. (qobuzMixes, the
    /// fourth ported Tauri-Home section, is static navigation tiles — no
    /// data field needed.)
    pub top_artists: Vec<crate::foryou::ArtistSlim>,
    /// "Most Played Albums" rail — top albums by local play count
    /// (`album_play_history`). Local, so built inline (no fetch). Feeds
    /// `HomeState.most-played-albums`; self-hides while empty.
    pub most_played_albums: Vec<crate::foryou::AlbumCard>,
}

#[derive(Clone)]
pub struct SectionData {
    /// The configurator section id this album carousel maps to. Lets the
    /// prefs-driven render loop key a pref id to its cached section data
    /// (Slice 5). Album-carousel sections only.
    pub id: DiscoverySectionId,
    pub title: String,
    /// Discover endpoint path for the "View all" page ("" = no full-list page).
    pub endpoint: String,
    pub albums: Vec<CardData>,
}

#[derive(Clone, Default)]
pub struct CardData {
    pub id: String,
    pub title: String,
    pub artist: String,
    /// Artist id for the clickable artist name; empty = not clickable
    /// (e.g. artist-page release cards, whose subtitle slot is the year).
    pub artist_id: String,
    pub genre: String,
    pub year: String,
    /// "hires" | "cd" | "" — drives the icon-only quality badge.
    pub quality_tier: String,
    /// "Hi-Res: 24-bit / 96 kHz" — shown when hovering the quality badge.
    pub quality_label: String,
    pub ribbon: String,
    pub ribbon_kind: String,
    pub artwork_url: String,
    // --- List-row extras (AlbumListRow); empty/default for grid-only data.
    /// "Album" | "EP" | "Single" | "Live" | "Compilation".
    pub release_type: String,
    /// "qobuz" | "local" | "" — the hideable SOURCE column.
    pub source: String,
    /// "24-bit / 96 kHz" — the bare detail line for QualityBadgeFull.
    pub quality_detail: String,
    /// Track count, as a display string ("" = unknown).
    pub track_count: String,
    /// Bare 4-digit year for the list-row YEAR column ("" = unknown).
    pub plain_year: String,
}

/// A single-cover playlist card for the Discover `qobuzPlaylists` row
/// (Home + Editor's Picks). Tauri's PlaylistCardLite renders name only — no
/// owner/subtitle/track-count — and a single cover, so we drop them too.
#[derive(Clone)]
pub struct PlaylistCardData {
    pub id: String,
    pub title: String,
    pub artwork_url: String, // rectangle || covers[0] || ""
    /// First tag's localized name — the UPPERCASE accent subtag on the card
    /// ("" = the playlist carries no tags).
    pub category: String,
    /// All tag slugs — the material for the client-side category filter (C).
    pub tags: Vec<String>,
}

/// A compact ranked item for the slim grid sections.
pub struct SlimData {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub rank: String,
    pub artwork_url: String,
}
