//! Trivial view-state setters that trigger a refresh.

use slint::ComponentHandle;

use super::QueueController;
use crate::QueueState;

impl QueueController {
    /// Move to the previous paginator page.
    pub fn prev_page(&self) {
        if let Ok(mut view) = self.view.lock() {
            view.page = view.page.saturating_sub(1);
        }
        self.refresh();
    }

    /// Move to the next paginator page.
    pub fn next_page(&self) {
        if let Ok(mut view) = self.view.lock() {
            view.page += 1;
        }
        self.refresh();
    }

    /// Switch the active tab (0 = Queue, 1 = History). Pushes the new index
    /// onto the Slint `QueueState.tab` property right away — `refresh()` is
    /// async, so without this the body never switched (the History tab read
    /// `tab == 1` but the property stayed 0, so clicking History did nothing).
    pub fn set_tab(&self, tab: i32) {
        if let Ok(mut view) = self.view.lock() {
            view.tab = tab;
        }
        if let Some(w) = self.weak.upgrade() {
            w.global::<QueueState>().set_tab(tab);
        }
        self.refresh();
    }

    /// Update the search query and re-filter the upcoming list. Changing
    /// the query resets the paginator to the first page.
    pub fn search_changed(&self, query: String) {
        if let Ok(mut view) = self.view.lock() {
            view.search = query;
            view.page = 0;
        }
        self.refresh();
    }
}
