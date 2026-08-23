/// The `DiscoverySectionId` union: the 18 remaining Tauri members
/// (`sectionPrefs.ts`; `radioStations` was retired with the Radio feature —
/// see REMOVAL-SPEC.md §6) plus the Slint-era `Pinned` (user-pinned
/// albums/artists/playlists — no Tauri counterpart) and the local
/// `MostPlayedAlbums` (top albums by local play count). `editorPicks` is
/// BOTH a tab and a section id (the "Albums
/// of the Week" section).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoverySectionId {
    NewReleases,
    PressAwards,
    QobuzPlaylists,
    RecentlyPlayedAlbums,
    ContinueListening,
    IdealDiscography,
    MostStreamed,
    ReleaseWatch,
    EditorPicks,
    Qobuzissimes,
    TopArtists,
    FavoriteAlbums,
    QobuzMixes,
    SimilarAlbums,
    RediscoverLibrary,
    EssentialsByGenre,
    ArtistsToFollow,
    ArtistSpotlight,
    Pinned,
    /// "Most Played Albums" — top albums by local play count
    /// (`qbz_app::settings::album_play_history`). Home + For You, default off.
    MostPlayedAlbums,
}

impl DiscoverySectionId {
    pub fn as_str(&self) -> &'static str {
        use DiscoverySectionId::*;
        match self {
            NewReleases => "newReleases",
            PressAwards => "pressAwards",
            QobuzPlaylists => "qobuzPlaylists",
            RecentlyPlayedAlbums => "recentlyPlayedAlbums",
            ContinueListening => "continueListening",
            IdealDiscography => "idealDiscography",
            MostStreamed => "mostStreamed",
            ReleaseWatch => "releaseWatch",
            EditorPicks => "editorPicks",
            Qobuzissimes => "qobuzissimes",
            TopArtists => "topArtists",
            FavoriteAlbums => "favoriteAlbums",
            QobuzMixes => "qobuzMixes",
            SimilarAlbums => "similarAlbums",
            RediscoverLibrary => "rediscoverLibrary",
            EssentialsByGenre => "essentialsByGenre",
            ArtistsToFollow => "artistsToFollow",
            ArtistSpotlight => "artistSpotlight",
            Pinned => "pinned",
            MostPlayedAlbums => "mostPlayedAlbums",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        use DiscoverySectionId::*;
        Some(match s {
            "newReleases" => NewReleases,
            "pressAwards" => PressAwards,
            "qobuzPlaylists" => QobuzPlaylists,
            "recentlyPlayedAlbums" => RecentlyPlayedAlbums,
            "continueListening" => ContinueListening,
            "idealDiscography" => IdealDiscography,
            "mostStreamed" => MostStreamed,
            "releaseWatch" => ReleaseWatch,
            "editorPicks" => EditorPicks,
            "qobuzissimes" => Qobuzissimes,
            "topArtists" => TopArtists,
            "favoriteAlbums" => FavoriteAlbums,
            "qobuzMixes" => QobuzMixes,
            "similarAlbums" => SimilarAlbums,
            "rediscoverLibrary" => RediscoverLibrary,
            "essentialsByGenre" => EssentialsByGenre,
            "artistsToFollow" => ArtistsToFollow,
            "artistSpotlight" => ArtistSpotlight,
            "pinned" => Pinned,
            "mostPlayedAlbums" => MostPlayedAlbums,
            _ => return None,
        })
    }
}
