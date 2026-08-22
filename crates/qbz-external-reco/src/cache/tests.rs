use std::time::{SystemTime, UNIX_EPOCH};

use super::{CacheLookup, RecoCache};

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-extreco-{tag}-{}-{nonce}", std::process::id()))
}

#[test]
fn positive_negative_and_miss() {
    let dir = tmp_dir("cache");
    let cache = RecoCache::open_at(&dir).expect("open");
    assert!(matches!(cache.get("k1"), CacheLookup::Miss));

    cache.put("k1", "track", Some("12345"));
    match cache.get("k1") {
        CacheLookup::Found(id) => assert_eq!(id, "12345"),
        _ => panic!("expected Found"),
    }

    cache.put("k2", "track", None);
    assert!(matches!(cache.get("k2"), CacheLookup::Negative));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn weekly_cache_per_week_and_stale_fallback() {
    let dir = tmp_dir("weekly");
    let cache = RecoCache::open_at(&dir).expect("open");

    // Empty until something is stored.
    assert!(cache.get_weekly("weekly-jams:mbid-A").is_none());
    assert!(cache.get_latest_weekly_for_patch("weekly-jams").is_none());

    // Store week A; exact-key hit + patch-latest both return it.
    cache.put_weekly("weekly-jams:mbid-A", "weekly-jams", "[\"A\"]");
    assert_eq!(cache.get_weekly("weekly-jams:mbid-A").as_deref(), Some("[\"A\"]"));
    assert_eq!(
        cache.get_latest_weekly_for_patch("weekly-jams").as_deref(),
        Some("[\"A\"]")
    );

    // A different week (new mbid) is a natural miss -> triggers a rebuild,
    // but the latest-for-patch fallback still serves week A.
    assert!(cache.get_weekly("weekly-jams:mbid-B").is_none());
    assert_eq!(
        cache.get_latest_weekly_for_patch("weekly-jams").as_deref(),
        Some("[\"A\"]")
    );

    // Patches are isolated from each other.
    assert!(cache.get_latest_weekly_for_patch("weekly-exploration").is_none());

    let _ = std::fs::remove_dir_all(dir);
}
