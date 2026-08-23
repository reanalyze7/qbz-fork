//! The stateful stack-mutation API built on top of `history.rs`.

use super::entry::NavEntry;
use super::history::{live_scroll, set_live_scroll, Entry, History, HISTORY};

/// Push a Search history entry, OR replace the cursor entry in place
/// when it is already a Search. Used by the live-search debounce so
/// quick keystrokes do not push one entry per character, while still
/// keeping the page reachable via back/forward at the final query.
pub fn push_or_replace_search(query: String) {
    HISTORY.with(|h| {
        let h = &mut *h.borrow_mut();
        match h.entries.get(h.cursor).map(|e| &e.nav) {
            Some(NavEntry::Search(_)) => {
                // Replace in place: same Search page, keep its scroll.
                let scroll = h.entries[h.cursor].scroll;
                h.entries.truncate(h.cursor + 1);
                h.entries[h.cursor] = Entry {
                    nav: NavEntry::Search(query),
                    scroll,
                };
            }
            _ => {
                if let Some(cur) = h.entries.get_mut(h.cursor) {
                    cur.scroll = live_scroll();
                }
                h.entries.truncate(h.cursor + 1);
                h.entries.push(Entry {
                    nav: NavEntry::Search(query),
                    scroll: 0.0,
                });
                h.cursor = h.entries.len() - 1;
            }
        }
    });
}

/// Record a fresh forward navigation, dropping any forward history. A
/// no-op when the destination already is the current entry, so repeated
/// clicks on the same page do not pile up.
pub fn record(entry: NavEntry) {
    let pushed = HISTORY.with(|h| {
        let h = &mut *h.borrow_mut();
        if h.entries.get(h.cursor).map(|e| &e.nav) == Some(&entry) {
            return false;
        }
        // Stamp the page we are leaving with its live scroll position.
        if let Some(cur) = h.entries.get_mut(h.cursor) {
            cur.scroll = live_scroll();
        }
        h.entries.truncate(h.cursor + 1);
        h.entries.push(Entry {
            nav: entry,
            scroll: 0.0,
        });
        h.cursor = h.entries.len() - 1;
        true
    });
    // A fresh page starts at the top; the new view will report its own
    // scroll as the user moves it.
    if pushed {
        set_live_scroll(0.0);
    }
}

/// Replace the whole history with a single root entry. Used by the OFFLINE
/// session entry (D12): the post-entry view IS the root, so back/forward
/// never lead to a phantom blocked Home.
pub fn reset_root(entry: NavEntry) {
    HISTORY.with(|h| {
        *h.borrow_mut() = History {
            entries: vec![Entry {
                nav: entry,
                scroll: 0.0,
            }],
            cursor: 0,
        };
    });
    set_live_scroll(0.0);
}

