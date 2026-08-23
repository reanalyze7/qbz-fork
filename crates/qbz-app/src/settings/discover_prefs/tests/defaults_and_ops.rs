use DiscoverySectionId::*;

use super::ids;
use crate::settings::discover_prefs::*;

// --- Group 1: default ordering + enabled flags ---

#[test]
fn defaults_match_spec_exactly() {
    let d = default_prefs();
    // home: 15 entries, first 8 ON (Tauri sectionPrefs.ts + Slint `pinned`
    // + the local mostPlayedAlbums, default off).
    assert_eq!(
        ids(&d.home),
        vec![
            NewReleases, PressAwards, QobuzPlaylists, RecentlyPlayedAlbums,
            ContinueListening, IdealDiscography, MostStreamed, Pinned,
            QobuzMixes, ReleaseWatch, EditorPicks, Qobuzissimes, TopArtists,
            FavoriteAlbums, MostPlayedAlbums,
        ]
    );
    assert_eq!(d.enabled_count(DiscoveryTab::Home), 8);
    assert!(d.is_enabled(DiscoveryTab::Home, MostStreamed));
    assert!(!d.is_enabled(DiscoveryTab::Home, Qobuzissimes));
    // editorPicks: 7 entries, all ON.
    assert_eq!(
        ids(&d.editor_picks),
        vec![NewReleases, EditorPicks, Qobuzissimes, PressAwards, MostStreamed, IdealDiscography, QobuzPlaylists]
    );
    assert_eq!(d.enabled_count(DiscoveryTab::EditorPicks), 7);
    // forYou: 13 entries (radioStations retired, REMOVAL-SPEC.md §6),
    // qobuzMixes first, pinned second; the 12 remaining Tauri+Slint ones
    // ON, mostPlayedAlbums (local addition) OFF.
    assert_eq!(d.for_you.len(), 13);
    assert_eq!(d.for_you[0].id, QobuzMixes);
    assert_eq!(d.for_you[1].id, Pinned);
    assert_eq!(d.for_you[12].id, MostPlayedAlbums);
    assert_eq!(d.enabled_count(DiscoveryTab::ForYou), 12);
}

// --- Group 4: move_section ---

#[test]
fn move_section_clamps_and_carries_enabled() {
    let mut d = default_prefs();
    // Up at index 0 is a no-op.
    d.move_section(DiscoveryTab::Home, NewReleases, -1);
    assert_eq!(d.home[0].id, NewReleases);
    // Down at last index is a no-op.
    let last = d.home.last().unwrap().id;
    d.move_section(DiscoveryTab::Home, last, 1);
    assert_eq!(d.home.last().unwrap().id, last);
    // Moving pressAwards (idx 1, enabled) up swaps with newReleases; enabled travels.
    d.move_section(DiscoveryTab::Home, PressAwards, -1);
    assert_eq!(d.home[0], SectionPref { id: PressAwards, enabled: true });
    assert_eq!(d.home[1], SectionPref { id: NewReleases, enabled: true });
    // Unknown id for the tab is a no-op (artistSpotlight not in home).
    let before = d.home.clone();
    d.move_section(DiscoveryTab::Home, ArtistSpotlight, 1);
    assert_eq!(d.home, before);
}

// --- Group 5: toggle (no floor) ---

#[test]
fn toggle_can_reach_zero_enabled() {
    let mut d = default_prefs();
    for p in d.editor_picks.clone() {
        d.toggle(DiscoveryTab::EditorPicks, p.id);
    }
    assert_eq!(d.enabled_count(DiscoveryTab::EditorPicks), 0);
    // Toggling back on works too.
    d.toggle(DiscoveryTab::EditorPicks, NewReleases);
    assert!(d.is_enabled(DiscoveryTab::EditorPicks, NewReleases));
}

#[test]
fn reset_tab_restores_defaults_only_for_that_tab() {
    let mut d = default_prefs();
    d.toggle(DiscoveryTab::Home, NewReleases);
    d.toggle(DiscoveryTab::ForYou, QobuzMixes);
    d.reset_tab(DiscoveryTab::Home);
    assert_eq!(d.home, default_prefs().home);
    // ForYou untouched by the Home reset.
    assert!(!d.is_enabled(DiscoveryTab::ForYou, QobuzMixes));
}
