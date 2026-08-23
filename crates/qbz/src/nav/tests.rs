use super::entry::NavEntry;
use super::history::{live_scroll, set_live_scroll, Entry, History, HISTORY};
use super::navigation::*;
use super::stepping::*;

fn reset() {
    HISTORY.with(|h| {
        *h.borrow_mut() = History {
            entries: vec![Entry {
                nav: NavEntry::Home,
                scroll: 0.0,
            }],
            cursor: 0,
        };
    });
    set_live_scroll(0.0);
}

/// Drop the scroll component for the assertions that only care about the
/// destination page.
fn nav_of(res: Option<(NavEntry, f32)>) -> Option<NavEntry> {
    res.map(|(e, _)| e)
}

#[test]
fn record_then_back_and_forward() {
    reset();
    assert!(!can_back());
    record(NavEntry::Album("1".into()));
    record(NavEntry::Artist("2".into()));
    assert!(can_back());
    assert!(!can_forward());
    assert_eq!(nav_of(go_back()), Some(NavEntry::Album("1".into())));
    assert_eq!(nav_of(go_back()), Some(NavEntry::Home));
    assert_eq!(nav_of(go_back()), None);
    assert_eq!(nav_of(go_forward()), Some(NavEntry::Album("1".into())));
}

#[test]
fn record_truncates_forward_history() {
    reset();
    record(NavEntry::Album("1".into()));
    record(NavEntry::Album("2".into()));
    go_back();
    record(NavEntry::Artist("3".into()));
    assert!(!can_forward());
    assert_eq!(nav_of(go_back()), Some(NavEntry::Album("1".into())));
}

#[test]
fn search_entry_round_trips_history() {
    reset();
    record(NavEntry::Search("metallica".into()));
    record(NavEntry::Album("5".into()));
    assert_eq!(
        nav_of(go_back()),
        Some(NavEntry::Search("metallica".into()))
    );
    assert_eq!(nav_of(go_back()), Some(NavEntry::Home));
}

#[test]
fn payload_free_and_id_carrying_round_trip_history() {
    reset();
    // A payload-free list entry, then two id-carrying detail entries —
    // the same shape the removed Purchases/PurchaseDetail pair exercised.
    record(NavEntry::Collections);
    record(NavEntry::Album("A".into()));
    record(NavEntry::Album("B".into()));
    // Back walks B → A → list → Home; forward returns to A then B.
    assert_eq!(nav_of(go_back()), Some(NavEntry::Album("A".into())));
    assert_eq!(nav_of(go_back()), Some(NavEntry::Collections));
    assert_eq!(nav_of(go_back()), Some(NavEntry::Home));
    assert_eq!(nav_of(go_forward()), Some(NavEntry::Collections));
    assert_eq!(nav_of(go_forward()), Some(NavEntry::Album("A".into())));
    assert_eq!(nav_of(go_forward()), Some(NavEntry::Album("B".into())));
}

#[test]
fn record_dedupes_current_entry() {
    reset();
    record(NavEntry::Album("1".into()));
    record(NavEntry::Album("1".into()));
    assert_eq!(nav_of(go_back()), Some(NavEntry::Home));
}

#[test]
fn scroll_is_stamped_on_leave_and_restored_on_return() {
    reset();
    // On Home, scroll down a bit, then navigate away.
    set_live_scroll(-420.0);
    record(NavEntry::Album("1".into()));
    // Fresh page starts at the top.
    assert_eq!(live_scroll(), 0.0);
    // Scroll the album page, then go back to Home.
    set_live_scroll(-90.0);
    let (entry, scroll) = go_back().expect("back to Home");
    assert_eq!(entry, NavEntry::Home);
    assert_eq!(scroll, -420.0);
    // Going forward returns to the album at its saved scroll.
    let (entry, scroll) = go_forward().expect("forward to album");
    assert_eq!(entry, NavEntry::Album("1".into()));
    assert_eq!(scroll, -90.0);
}
