use super::*;

fn item(kind: &str, id: &str, title: &str) -> PinnedItem {
    PinnedItem {
        kind: kind.to_string(),
        id: id.to_string(),
        title: title.to_string(),
        subtitle: format!("{title} subtitle"),
        artwork_url: String::new(),
        pinned_at: 0, // ignored on write; the service stamps now
    }
}

/// One combined lifecycle test covering the full service surface:
/// pin+check, kind isolation, upsert-replaces, ordered list roundtrip
/// with NULL-tolerant fields, count/keys_snapshot, unpin (absent = Ok).
#[test]
fn lifecycle() {
    let s = PinnedItemsService::new_in_memory().expect("svc");

    // Fresh store: nothing pinned.
    assert!(!s.is_pinned("album", "abc123"));
    assert_eq!(s.count(), 0);
    assert!(s.keys_snapshot().is_empty());
    assert!(s.list().unwrap().is_empty());

    // Pin + check.
    s.pin(&item("album", "abc123", "First Album")).unwrap();
    assert!(s.is_pinned("album", "abc123"));
    assert!(!s.is_pinned("album", "zzz999"));

    // Kind isolation: pinning album id X does not pin playlist id X.
    assert!(!s.is_pinned("playlist", "abc123"));
    s.pin(&item("playlist", "abc123", "Same-Id Playlist"))
        .unwrap();
    assert!(s.is_pinned("playlist", "abc123"));
    assert_eq!(s.count(), 2);

    // Upsert replaces the display snapshot, keeps one row.
    s.pin(&item("album", "abc123", "Renamed Album")).unwrap();
    assert_eq!(s.count(), 2);
    let all = s.list().unwrap();
    assert_eq!(all.len(), 2);
    let renamed = all
        .iter()
        .find(|i| i.kind == "album" && i.id == "abc123")
        .expect("upserted row present");
    assert_eq!(renamed.title, "Renamed Album");
    assert_eq!(renamed.subtitle, "Renamed Album subtitle");
    assert_eq!(renamed.artwork_url, "");

    // Ordered list roundtrip: pinned_at is stamped and non-increasing
    // (newest first). Same-second pins tie, so assert the DESC property
    // rather than a strict order between them.
    s.pin(&item("artist", "42", "An Artist")).unwrap();
    let all = s.list().unwrap();
    assert_eq!(all.len(), 3);
    assert!(all.windows(2).all(|w| w[0].pinned_at >= w[1].pinned_at));
    assert!(all.iter().all(|i| i.pinned_at > 0));

    // Snapshot mirrors the set.
    let keys = s.keys_snapshot();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&("artist".to_string(), "42".to_string())));

    // Unpin: removed from check + list; absent is Ok, not error.
    s.unpin("album", "abc123").unwrap();
    assert!(!s.is_pinned("album", "abc123"));
    assert!(s.is_pinned("playlist", "abc123")); // other kind untouched
    assert_eq!(s.count(), 2);
    s.unpin("album", "nope").unwrap(); // absent -> Ok
    assert_eq!(s.list().unwrap().len(), 2);
}
