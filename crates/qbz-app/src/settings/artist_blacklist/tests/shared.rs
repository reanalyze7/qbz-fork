use super::svc;

#[test]
fn shared_enabled_flag_gates_both_axes() {
    let s = svc();
    s.add(1, "Artist", None).unwrap();
    s.add_album("alb", "Album", "Artist", "", None).unwrap();
    s.set_enabled(false).unwrap();
    assert!(!s.is_blacklisted(1)); // both off
    assert!(!s.is_album_blacklisted("alb"));
    assert_eq!(s.count(), 1); // counts ignore the flag
    assert_eq!(s.album_count(), 1);
    s.set_enabled(true).unwrap();
    assert!(s.is_blacklisted(1)); // both back on
    assert!(s.is_album_blacklisted("alb"));
}

#[test]
fn axes_are_independent() {
    let s = svc();
    s.add_album("alb", "Album", "Artist", "", None).unwrap();
    assert_eq!(s.album_count(), 1);
    assert_eq!(s.count(), 0); // blocking an album leaves the artist set empty

    s.add(7, "Artist", None).unwrap();
    s.clear_all_albums().unwrap();
    assert_eq!(s.album_count(), 0);
    assert_eq!(s.count(), 1); // clear_all_albums leaves artist rows intact
    assert!(s.is_blacklisted(7));

    s.add_album("alb2", "A2", "Ar", "", None).unwrap();
    s.clear_all().unwrap();
    assert_eq!(s.count(), 0);
    assert_eq!(s.album_count(), 1); // clear_all (artists) leaves albums intact
}
