use super::svc;

#[test]
fn album_add_and_check() {
    let s = svc();
    s.add_album("abc123", "Bogus Anthrax", "Anthrax", "http://c", None)
        .unwrap();
    assert!(s.is_album_blacklisted("abc123"));
    assert!(!s.is_album_blacklisted("zzz999"));
}

#[test]
fn album_remove_is_not_error_when_absent() {
    let s = svc();
    s.add_album("a", "T", "Ar", "", None).unwrap();
    s.remove_album("a").unwrap();
    assert!(!s.is_album_blacklisted("a"));
    s.remove_album("nope").unwrap(); // absent -> Ok
}

#[test]
fn album_get_all_sorted_by_title_with_fields_roundtrip() {
    let s = svc();
    s.add_album("2", "zeta", "Z Artist", "http://z", Some("n"))
        .unwrap();
    s.add_album("1", "Alpha", "A Artist", "", None).unwrap();
    let all = s.get_all_albums().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].album_title, "Alpha"); // case-insensitive asc
    assert_eq!(all[1].album_title, "zeta");
    assert_eq!(all[0].artist_name, "A Artist");
    assert_eq!(all[0].cover_url, "");
    assert_eq!(all[1].cover_url, "http://z");
    assert_eq!(all[1].notes.as_deref(), Some("n"));
}

#[test]
fn album_upsert_replaces() {
    let s = svc();
    s.add_album("5", "Old", "A", "u1", Some("n")).unwrap();
    s.add_album("5", "New", "B", "u2", None).unwrap();
    let all = s.get_all_albums().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].album_title, "New");
    assert_eq!(all[0].cover_url, "u2");
    assert_eq!(all[0].notes, None);
}
