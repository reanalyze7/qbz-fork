//! The `NavEntry` enum — every navigable page/tab in the app. Pure data,
//! no logic.

/// One navigable destination.
///
/// `Serialize`/`Deserialize` back the "Startup page = where you left off"
/// restore: the current entry is persisted as JSON in `ui_prefs.last_nav` and
/// reconstructed at launch (every payload is a plain String/u64/Vec<String>,
/// so the derive round-trips the whole enum).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NavEntry {
    Home,
    /// A Discover tab page ("home" | "editorPicks" | "forYou"). Each
    /// tab is its own history entry so back/forward moves between the
    /// three Discover pages.
    Discover {
        tab: String,
    },
    /// A Library > Favorites tab page ("tracks" | "albums" | "artists" |
    /// "playlists" | "labels"). Each tab is its own history entry so
    /// back/forward moves between the favorites pages, mirroring Discover.
    Favorites {
        tab: String,
    },
    /// A Local Library browse tab page ("tracks" | "folders" | "albums" |
    /// "artists"). Each tab is its own history entry so back/forward moves
    /// between the Local Library tabs, mirroring Favorites / Discover.
    LocalLibrary {
        tab: String,
    },
    /// A Discover "View all" full-list page — one album module
    /// (new releases, qobuzissimes, ...) opened from a Carousel's
    /// "View all" link. Carries the /discover/<x> endpoint + the
    /// section title used as the page heading.
    DiscoverBrowse {
        endpoint: String,
        title: String,
    },
    /// RecentAlbumsView — the full "Recently Played Albums" listing reached
    /// from the Home rail's "View all". Renders the LOCAL play-history album
    /// store (crate::recently), reloaded on every navigation, so the entry
    /// carries no payload.
    RecentAlbums,
    /// PlaylistBrowseView — the Qobuz Playlists "View all" full-list page
    /// reached from the Home / Editor's Picks rail. Tag / search / view-mode
    /// are session state in PlaylistBrowseState, so no payload.
    PlaylistBrowse,
    /// MostPlayedAlbumsView — the full "Most Played Albums" listing, ranked by
    /// local play count (album_play_history), reloaded on every navigation.
    MostPlayedAlbums,
    /// A Qobuz mix detail page ("daily" | "weekly" | "fav" | "top").
    Mix {
        kind: String,
    },
    /// A playlist detail page; the string is the playlist id.
    Playlist(String),
    /// The Playlist Manager — the full playlist + folder organization
    /// surface (Tauri's PlaylistManagerView). Toolbar state
    /// (filter/sort/view/folder-mode) is session-scoped in the
    /// controller, so the entry carries no payload.
    PlaylistManager,
    /// The Offline Cache Manager — the manage-downloads surface (Tauri's
    /// OfflineCacheManagerView). Session-scoped; no payload.
    OfflineManager,
    /// The Artist Blacklist Manager — the manage-blacklist surface (Tauri's
    /// BlacklistManagerView). Reached from the Settings content-filtering row.
    /// Session-scoped (the search query lives in the controller); no payload.
    BlacklistManager,
    /// The My QBZ > Mixtapes index grid (read-only in this slice). Toolbar
    /// state (sort/view/search) is session-scoped in the controller, so the
    /// entry carries no payload.
    Mixtapes,
    /// The My QBZ > Collections index grid (read-only in this slice). Same
    /// session-scoped toolbar; no payload.
    Collections,
    /// A Mixtape / Collection / Artist-Collection DETAIL page (read-only in
    /// this slice); the string is the collection id. Mirrors `Album` /
    /// `Playlist` — the in-detail toolbar state (search / sort / type-filter /
    /// view-mode) is session-scoped in the controller, so the entry carries
    /// only the id.
    MixtapeDetail(String),
    Album(String),
    /// A Local Library album detail page (dedicated view, separate from the
    /// Qobuz Album view); the string is the metadata group key.
    LocalAlbum(String),
    Artist(String),
    Settings,
    /// A search results page; the string is the query.
    Search(String),
    /// MusicianPageView — opened by the artist network sidebar when
    /// the resolved musician does not have a confident Qobuz artist
    /// match. Carries the musician name + the role for the
    /// appearances query.
    Musician {
        name: String,
        role: String,
    },
    /// LabelView landing — the rich label page (header + popular tracks +
    /// releases / critics / playlists / artists / more-labels carousels).
    /// Reached by clicking a label anywhere. Carries the id + name fallback.
    Label {
        id: u64,
        name: String,
    },
    /// LabelReleasesView — the "See all releases" sub-view reached from the
    /// landing's Releases carousel. Carries the label id + name fallback.
    LabelReleases {
        id: u64,
        name: String,
    },
    /// ArtistReleasesView — the dedicated discography listing for one
    /// release bucket, reached via "See discography" on the artist page.
    ArtistReleases {
        id: String,
        name: String,
        release_type: String,
    },
    /// ArtistsByLocationView — opened by the Origin section's
    /// location link. Carries the full scene-discovery payload.
    Location {
        mbid: String,
        area_id: String,
        area_name: String,
        country: String,
        genres: Vec<String>,
        tags: Vec<String>,
    },
}
