use super::model::{pref, DiscoverPrefs};
use super::section_id::DiscoverySectionId;

/// The exact `DEFAULT_PREFS` from `sectionPrefs.ts`.
pub fn default_prefs() -> DiscoverPrefs {
    use DiscoverySectionId::*;
    DiscoverPrefs {
        // home: first 8 ON, the rest OFF (the Tauri sectionPrefs.ts:63-77
        // list plus the Slint-era `pinned` — enabled by default; its arm
        // self-hides while the user has no pins — and the local
        // `mostPlayedAlbums` addition, default off). All 13 Tauri ids render
        // on Home since #566 completed the port: qobuzMixes / releaseWatch /
        // topArtists / favoriteAlbums were genuine Tauri-Home sections whose
        // Slint render arms + data pipelines were missing.
        home: vec![
            pref(NewReleases, true),
            pref(PressAwards, true),
            pref(QobuzPlaylists, true),
            pref(RecentlyPlayedAlbums, true),
            pref(ContinueListening, true),
            pref(IdealDiscography, true),
            pref(MostStreamed, true),
            pref(Pinned, true),
            pref(QobuzMixes, false),
            pref(ReleaseWatch, false),
            pref(EditorPicks, false),
            pref(Qobuzissimes, false),
            pref(TopArtists, false),
            pref(FavoriteAlbums, false),
            pref(MostPlayedAlbums, false),
        ],
        // editorPicks: all ON.
        editor_picks: vec![
            pref(NewReleases, true),
            pref(EditorPicks, true),
            pref(Qobuzissimes, true),
            pref(PressAwards, true),
            pref(MostStreamed, true),
            pref(IdealDiscography, true),
            pref(QobuzPlaylists, true),
        ],
        // forYou: all ON, qobuzMixes first, pinned right after (near the top —
        // it is the user's own curation; self-hides while empty).
        for_you: vec![
            pref(QobuzMixes, true),
            pref(Pinned, true),
            pref(ReleaseWatch, true),
            pref(ContinueListening, true),
            pref(RecentlyPlayedAlbums, true),
            pref(TopArtists, true),
            pref(FavoriteAlbums, true),
            pref(SimilarAlbums, true),
            pref(RediscoverLibrary, true),
            pref(EssentialsByGenre, true),
            pref(ArtistsToFollow, true),
            pref(ArtistSpotlight, true),
            pref(MostPlayedAlbums, false),
        ],
        show_recommendations: true,
        reco_cache_ttl_hours: 48,
    }
}
