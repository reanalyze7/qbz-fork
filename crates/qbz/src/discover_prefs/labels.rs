//! id -> render kind / i18n label key lookup tables.

use qbz_app::settings::discover_prefs::DiscoverySectionId;

/// Coarse render family. Used by the one tab-dependent Home arm: `mostStreamed`
/// renders as a slim grid on Home but an album carousel on Editor's Picks. The
/// rest of the dispatch is by id; this is harmless metadata for those.
pub fn render_kind(id: DiscoverySectionId) -> &'static str {
    use DiscoverySectionId::*;
    match id {
        // Album carousels (Home / Editor share the Carousel component).
        NewReleases | PressAwards | IdealDiscography | EditorPicks | Qobuzissimes => "albumCarousel",
        // mostStreamed is overridden per tab in `home::tab_descriptors`; this is
        // its Home default.
        MostStreamed => "slimGrid",
        QobuzPlaylists => "playlistCarousel",
        RecentlyPlayedAlbums => "albumCarousel",
        ContinueListening => "slimGrid",
        QobuzMixes => "qobuzMixes",
        TopArtists | ArtistsToFollow => "artistCarousel",
        ArtistSpotlight => "spotlight",
        Pinned => "pinnedCarousel",
        ReleaseWatch | FavoriteAlbums | MostPlayedAlbums | SimilarAlbums | RediscoverLibrary
        | EssentialsByGenre => "albumCarousel",
    }
}

/// id -> Tauri i18n key (frontend concern, NOT in the headless prefs crate per
/// ADR-006). Resolved to a string in Rust because Slint `@tr` needs a literal
/// key. Returns the English label today (the Slint gettext pipeline is unwired,
/// so labels are plain Rust strings — consistent with every other Slint section
/// title). When gettext lands this swaps to an MO lookup with NO `.slint` change.
/// The keys are kept verbatim (with their real, mixed `home.*` / `discover.*` /
/// `discovery.*` namespaces) so the lookup ports 1:1 when the pipeline arrives.
pub fn label_for(id: DiscoverySectionId) -> &'static str {
    use DiscoverySectionId::*;
    // Returns the `mark`ed English literal so the extractor registers the
    // msgid here; the single `t(...)` lookup happens at the consumer
    // (`push_config_rows`). This translates each label exactly once.
    match id {
        NewReleases => qbz_i18n::mark("New Releases"), // home.newReleases
        PressAwards => qbz_i18n::mark("Press Accolades"), // home.pressAwards
        QobuzPlaylists => qbz_i18n::mark("Qobuz Playlists"), // home.qobuzPlaylists
        RecentlyPlayedAlbums => qbz_i18n::mark("Recently Played"), // home.recentlyPlayed
        ContinueListening => qbz_i18n::mark("Continue Listening"), // home.continueListening
        IdealDiscography => qbz_i18n::mark("Ideal Discography"), // discover.idealDiscography
        MostStreamed => qbz_i18n::mark("Most Streamed"), // home.mostStreamed
        ReleaseWatch => qbz_i18n::mark("Release Watch"), // home.releaseWatch
        EditorPicks => qbz_i18n::mark("Albums of the Week"), // home.editorPicks
        Qobuzissimes => qbz_i18n::mark("Qobuzissimes"), // home.qobuzissimes
        TopArtists => qbz_i18n::mark("Your Top Artists"), // home.yourTopArtists
        FavoriteAlbums => qbz_i18n::mark("Library Albums"), // home.favoriteAlbums
        QobuzMixes => qbz_i18n::mark("Qobuz Mixes"), // home.qobuzMixes
        SimilarAlbums => qbz_i18n::mark("More From Your Library"), // discovery.similarAlbums
        RediscoverLibrary => qbz_i18n::mark("Rediscover Your Library"), // discovery.rediscoverLibrary
        EssentialsByGenre => qbz_i18n::mark("Essentials by Genre"), // discovery.essentialsByGenre
        ArtistsToFollow => qbz_i18n::mark("Artists to Follow"), // discovery.artistsToFollow
        ArtistSpotlight => qbz_i18n::mark("Artist Spotlight"), // discovery.artistSpotlight
        Pinned => qbz_i18n::mark("Pinned"), // Slint-era section, no Tauri key
        MostPlayedAlbums => qbz_i18n::mark("Most Played Albums"), // local: most-played rail
    }
}
