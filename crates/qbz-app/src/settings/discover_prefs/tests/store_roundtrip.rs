use rusqlite::params;
use DiscoverySectionId::*;

use super::unique_test_dir;
use crate::settings::discover_prefs::*;

// --- Group 6: store round-trip + corruption ---

#[test]
fn store_roundtrip_and_corruption_recovery() {
    let dir = unique_test_dir("discover-prefs");
    {
        let store = DiscoverPrefsStore::new_at(&dir).expect("open store");
        // Fresh store returns defaults.
        assert_eq!(store.load(), default_prefs());
        // Mutate + save.
        let mut prefs = store.load();
        prefs.toggle(DiscoveryTab::Home, QobuzPlaylists);
        prefs.move_section(DiscoveryTab::Home, MostStreamed, -1);
        store.save(&prefs).expect("save");
        // Same-handle load is identity.
        assert_eq!(store.load(), prefs);
    }
    // Reopen -> persisted survives.
    {
        let store = DiscoverPrefsStore::new_at(&dir).expect("reopen");
        let prefs = store.load();
        assert!(!prefs.is_enabled(DiscoveryTab::Home, QobuzPlaylists));
        assert_eq!(prefs.home[0].id, NewReleases); // mostStreamed moved up from idx 6 to 5, not to 0
    }
    // Corrupt the blob -> load recovers to defaults.
    {
        let store = DiscoverPrefsStore::new_at(&dir).expect("reopen2");
        store
            .conn
            .execute(
                "UPDATE discover_prefs SET prefs_json = ?1 WHERE id = 1",
                params!["{not valid json"],
            )
            .expect("corrupt");
        assert_eq!(store.load(), default_prefs());
    }
    let _ = std::fs::remove_dir_all(dir);
}
