//! The `Entry`/`History` storage primitives, the `thread_local!` statics,
//! and the live-scroll bookkeeping.

use std::cell::{Cell, RefCell};

use super::entry::NavEntry;

/// One slot in the history stack: where we went, and how far it was
/// scrolled when we last left it.
#[derive(Clone, Debug)]
pub(super) struct Entry {
    pub(super) nav: NavEntry,
    /// Saved Flickable `viewport-y` (logical px; 0 at top, negative when
    /// scrolled down — Slint's convention).
    pub(super) scroll: f32,
}

pub(super) struct History {
    pub(super) entries: Vec<Entry>,
    /// Index of the entry currently shown.
    pub(super) cursor: usize,
}

thread_local! {
    pub(super) static HISTORY: RefCell<History> = RefCell::new(History {
        entries: vec![Entry { nav: NavEntry::Home, scroll: 0.0 }],
        cursor: 0,
    });
    /// Live `viewport-y` of the scroll container currently on screen, kept
    /// fresh by the mounted view via [`set_live_scroll`]. Read when leaving
    /// a page so its entry can be stamped without per-call-site plumbing.
    static LIVE_SCROLL: Cell<f32> = const { Cell::new(0.0) };
}

/// Record the on-screen scroll container's current `viewport-y`. Wired to
/// `NavState.report-scroll`, fired from the view's `changed viewport-y`.
pub fn set_live_scroll(y: f32) {
    LIVE_SCROLL.with(|s| s.set(y));
}

/// The entry currently shown (history top at the cursor). Used to persist the
/// "where you left off" startup destination.
pub fn current() -> Option<NavEntry> {
    HISTORY.with(|h| {
        let h = h.borrow();
        h.entries.get(h.cursor).map(|e| e.nav.clone())
    })
}

pub(super) fn live_scroll() -> f32 {
    LIVE_SCROLL.with(|s| s.get())
}
