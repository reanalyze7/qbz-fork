//! Back/forward stack stepping.

use super::entry::NavEntry;
use super::history::{live_scroll, set_live_scroll, HISTORY};

/// Step back; returns the entry that is now current plus its saved scroll
/// position, or `None` at the start of the stack.
pub fn go_back() -> Option<(NavEntry, f32)> {
    let res = HISTORY.with(|h| {
        let h = &mut *h.borrow_mut();
        if h.cursor == 0 {
            return None;
        }
        // Stamp the page we are leaving before stepping away.
        if let Some(cur) = h.entries.get_mut(h.cursor) {
            cur.scroll = live_scroll();
        }
        h.cursor -= 1;
        h.entries.get(h.cursor).map(|e| (e.nav.clone(), e.scroll))
    });
    if let Some((_, scroll)) = &res {
        set_live_scroll(*scroll);
    }
    res
}

/// Step forward; returns the entry that is now current plus its saved scroll
/// position, or `None` at the end of the stack.
pub fn go_forward() -> Option<(NavEntry, f32)> {
    let res = HISTORY.with(|h| {
        let h = &mut *h.borrow_mut();
        if h.cursor + 1 >= h.entries.len() {
            return None;
        }
        if let Some(cur) = h.entries.get_mut(h.cursor) {
            cur.scroll = live_scroll();
        }
        h.cursor += 1;
        h.entries.get(h.cursor).map(|e| (e.nav.clone(), e.scroll))
    });
    if let Some((_, scroll)) = &res {
        set_live_scroll(*scroll);
    }
    res
}

/// Whether a back step is available.
pub fn can_back() -> bool {
    HISTORY.with(|h| h.borrow().cursor > 0)
}

/// Whether a forward step is available.
pub fn can_forward() -> bool {
    HISTORY.with(|h| {
        let h = h.borrow();
        h.cursor + 1 < h.entries.len()
    })
}
