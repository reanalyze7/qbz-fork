use super::svc;

#[test]
fn add_and_check() {
    let s = svc();
    s.add(123, "Test Artist", None).unwrap();
    assert!(s.is_blacklisted(123));
    assert!(!s.is_blacklisted(456));
}

#[test]
fn remove_is_not_error_when_absent() {
    let s = svc();
    s.add(1, "A", None).unwrap();
    s.remove(1).unwrap();
    assert!(!s.is_blacklisted(1));
    s.remove(999).unwrap(); // absent -> Ok, not error
}

#[test]
fn disabled_short_circuits_even_with_row() {
    let s = svc();
    s.add(1, "A", None).unwrap();
    s.set_enabled(false).unwrap();
    assert!(!s.is_blacklisted(1)); // disabled => false even though row exists
    assert_eq!(s.count(), 1); // count ignores the enabled flag
    s.set_enabled(true).unwrap();
    assert!(s.is_blacklisted(1)); // re-enable restores instantly
}

#[test]
fn get_all_sorted_by_name_nocase_with_notes_roundtrip() {
    let s = svc();
    s.add(2, "zeta", Some("note-z".into())).unwrap();
    s.add(1, "Alpha", None).unwrap();
    let all = s.get_all().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].artist_name, "Alpha"); // case-insensitive asc
    assert_eq!(all[1].artist_name, "zeta");
    assert_eq!(all[1].notes.as_deref(), Some("note-z"));
    assert_eq!(all[0].notes, None);
}

#[test]
fn upsert_replaces_name_and_notes() {
    let s = svc();
    s.add(5, "Old", Some("n".into())).unwrap();
    s.add(5, "New", None).unwrap(); // INSERT OR REPLACE
    let all = s.get_all().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].artist_name, "New");
    assert_eq!(all[0].notes, None);
}

#[test]
fn clear_all_keeps_settings() {
    let s = svc();
    s.add(1, "A", None).unwrap();
    s.set_enabled(false).unwrap();
    s.clear_all().unwrap();
    assert_eq!(s.count(), 0);
    assert!(!s.is_enabled()); // clear_all does NOT touch enabled
}
