//! Toolbar actions: load + the three filter mutators.

use slint::ComponentHandle;

use crate::{AppWindow, OfflineManagerState};

use super::filters::filters;
use super::rebuild::rebuild;

/// Load (or refresh) the manager. Marks loading, then rebuilds.
pub fn load(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<OfflineManagerState>().set_loading(true);
    });
    handle.spawn(rebuild(weak));
}

pub fn select_artist(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle, name: String) {
    if let Ok(mut f) = filters().lock() {
        f.selected_artist = name;
    }
    handle.spawn(rebuild(weak));
}

pub fn set_sort(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle, index: i32) {
    if let Ok(mut f) = filters().lock() {
        f.sort = index;
    }
    handle.spawn(rebuild(weak));
}

pub fn toggle_failed(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    if let Ok(mut f) = filters().lock() {
        f.show_only_failed = !f.show_only_failed;
    }
    handle.spawn(rebuild(weak));
}
