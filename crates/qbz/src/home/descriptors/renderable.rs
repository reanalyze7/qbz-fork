//! The set of section ids the Home / Editor's Picks repeater actually has a
//! rendering arm for.

use qbz_app::settings::discover_prefs::DiscoverySectionId;

/// SINGLE SOURCE OF TRUTH for what the Home / Editor's Picks repeater can
/// actually render (#566): the section ids `HomeView.slint`'s delegate
/// if-chain has arms for. `descriptors_for` drops any enabled id NOT in this
/// set, so a stale persisted pref (e.g. qobuzMixes / releaseWatch / topArtists,
/// removed from the Home defaults 2026-07) can never emit an armless
/// descriptor again — an enabled section that renders nothing. Belt and
/// suspenders with `reconcile_list` (qbz-app), which already scrubs ids
/// absent from the tab defaults at load time. Extend this list IN THE SAME
/// CHANGE that adds a new arm to HomeView.slint.
pub(super) const HOME_RENDERABLE: &[DiscoverySectionId] = &[
    DiscoverySectionId::NewReleases,
    DiscoverySectionId::PressAwards,
    DiscoverySectionId::IdealDiscography,
    DiscoverySectionId::EditorPicks,
    DiscoverySectionId::Qobuzissimes,
    DiscoverySectionId::MostStreamed,
    DiscoverySectionId::QobuzPlaylists,
    DiscoverySectionId::RecentlyPlayedAlbums,
    DiscoverySectionId::ContinueListening,
    DiscoverySectionId::FavoriteAlbums,
    DiscoverySectionId::QobuzMixes,
    DiscoverySectionId::ReleaseWatch,
    DiscoverySectionId::TopArtists,
    DiscoverySectionId::Pinned,
    DiscoverySectionId::MostPlayedAlbums,
];
