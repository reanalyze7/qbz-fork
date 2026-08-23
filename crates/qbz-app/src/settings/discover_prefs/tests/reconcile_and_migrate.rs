use serde_json::json;
use DiscoverySectionId::*;

use super::ids;
use crate::settings::discover_prefs::*;

// --- Group 2: reconcile_list ---

#[test]
fn reconcile_none_returns_fallback() {
    let fb = default_prefs().home;
    assert_eq!(reconcile_list(None, &fb), fb);
}

#[test]
fn reconcile_drops_unknown_dedupes_coerces_and_appends_missing() {
    let fb = default_prefs().home;
    // Persisted: a reordered + partial list with an unknown id, a dupe,
    // a non-bool enabled, and an id not valid for this tab.
    let persisted = vec![
        json!({ "id": "mostStreamed", "enabled": false }),
        json!({ "id": "totallyUnknown", "enabled": true }),
        json!({ "id": "newReleases", "enabled": true }),
        json!({ "id": "newReleases", "enabled": false }), // dupe -> dropped
        json!({ "id": "artistSpotlight", "enabled": true }), // not in home defaults -> dropped
        json!({ "id": "pressAwards" }),                    // missing enabled -> false
    ];
    let out = reconcile_list(Some(&persisted), &fb);
    // Order: valid persisted first (mostStreamed, newReleases, pressAwards),
    // then the remaining home defaults in default order.
    assert_eq!(out[0], SectionPref { id: MostStreamed, enabled: false });
    assert_eq!(out[1], SectionPref { id: NewReleases, enabled: true });
    assert_eq!(out[2], SectionPref { id: PressAwards, enabled: false });
    // No unknown / cross-tab id leaked in.
    assert!(!ids(&out).contains(&ArtistSpotlight));
    // Every home default id is present exactly once.
    let mut got = ids(&out);
    got.sort_by_key(|i| i.as_str());
    let mut want = ids(&fb);
    want.sort_by_key(|i| i.as_str());
    assert_eq!(got, want);
    assert_eq!(out.len(), fb.len());
}

// --- Group 3: migrate (3 branches) ---

#[test]
fn migrate_legacy_array_is_home_only() {
    let legacy = json!([
        { "id": "qobuzPlaylists", "enabled": false },
        { "id": "newReleases", "enabled": true },
    ]);
    let m = DiscoverPrefs::migrate(&legacy);
    // Home reconciled from the array (qobuzPlaylists first, disabled).
    assert_eq!(m.home[0], SectionPref { id: QobuzPlaylists, enabled: false });
    assert_eq!(m.home[1], SectionPref { id: NewReleases, enabled: true });
    // The other two tabs are raw defaults.
    assert_eq!(m.editor_picks, default_prefs().editor_picks);
    assert_eq!(m.for_you, default_prefs().for_you);
}

#[test]
fn migrate_object_reconciles_all_three_tabs() {
    let obj = json!({
        "home": [{ "id": "newReleases", "enabled": false }],
        // editorPicks missing -> defaults; forYou present but empty -> defaults appended.
        "forYou": [],
    });
    let m = DiscoverPrefs::migrate(&obj);
    assert_eq!(m.home[0], SectionPref { id: NewReleases, enabled: false });
    assert_eq!(m.home.len(), default_prefs().home.len()); // missing appended
    assert_eq!(m.editor_picks, default_prefs().editor_picks);
    assert_eq!(m.for_you, default_prefs().for_you); // empty array -> all defaults appended
}

#[test]
fn migrate_garbage_returns_defaults() {
    assert_eq!(DiscoverPrefs::migrate(&json!(null)), default_prefs());
    assert_eq!(DiscoverPrefs::migrate(&json!(42)), default_prefs());
    assert_eq!(DiscoverPrefs::migrate(&json!("nope")), default_prefs());
}
