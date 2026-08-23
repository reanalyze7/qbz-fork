//! Mutation handlers (mutate -> persist -> re-push -> re-render).

use qbz_app::settings::discover_prefs::{DiscoverySectionId, DiscoveryTab};
use slint::ComponentHandle;

use super::{current, persist, push_config_rows, push_descriptors, PREFS};
use crate::artwork::{spawn_loads, ImageCache};
use crate::{AppWindow, DiscoverState};

/// After any mutation: re-push the descriptor lists (visibility + order),
/// refresh the live modal rows for the active tab, and re-render Home / Editor
/// from the cache (For You's data already lives in ForYouState — descriptors are
/// its sole driver). Returns artwork jobs to re-fire for newly-shown Home album
/// sections (mirrors `select_tab`'s job return); empty for For You.
fn apply_after_mutation(window: &AppWindow, mutated: DiscoveryTab) -> Vec<crate::artwork::ArtworkJob> {
    let prefs = current();
    push_descriptors(window, &prefs);
    if let Some(active) =
        DiscoveryTab::from_key(window.global::<DiscoverState>().get_active_tab().as_str())
    {
        push_config_rows(window, &prefs, active);
    }
    match mutated {
        DiscoveryTab::Home | DiscoveryTab::EditorPicks => {
            crate::home::rerender_active_tab(window, &prefs)
        }
        // For You: descriptor list is the sole driver; data already in ForYouState.
        DiscoveryTab::ForYou => Vec::new(),
    }
}

pub fn on_open_configurator(window: &AppWindow) {
    let prefs = current();
    if let Some(active) =
        DiscoveryTab::from_key(window.global::<DiscoverState>().get_active_tab().as_str())
    {
        push_config_rows(window, &prefs, active);
    }
    window.global::<DiscoverState>().set_configurator_open(true);
}

pub fn on_close_configurator(window: &AppWindow) {
    window.global::<DiscoverState>().set_configurator_open(false);
}

pub fn on_toggle(window: &AppWindow, tab: &str, id: &str, cache: &ImageCache) {
    let (Some(tab), Some(id)) = (DiscoveryTab::from_key(tab), DiscoverySectionId::from_str(id))
    else {
        return;
    };
    if let Some(p) = PREFS.lock().unwrap().as_mut() {
        p.toggle(tab, id);
    }
    persist();
    let jobs = apply_after_mutation(window, tab);
    spawn_loads(jobs, window.as_weak(), cache.clone());
}

pub fn on_move(window: &AppWindow, tab: &str, id: &str, dir: i32, cache: &ImageCache) {
    let (Some(tab), Some(id)) = (DiscoveryTab::from_key(tab), DiscoverySectionId::from_str(id))
    else {
        return;
    };
    let dir = dir.clamp(-1, 1) as i8;
    if let Some(p) = PREFS.lock().unwrap().as_mut() {
        p.move_section(tab, id, dir);
    }
    persist();
    let jobs = apply_after_mutation(window, tab);
    spawn_loads(jobs, window.as_weak(), cache.clone());
}

pub fn on_reset(window: &AppWindow, tab: &str, cache: &ImageCache) {
    let Some(tab) = DiscoveryTab::from_key(tab) else {
        return;
    };
    if let Some(p) = PREFS.lock().unwrap().as_mut() {
        p.reset_tab(tab);
    }
    persist();
    let jobs = apply_after_mutation(window, tab);
    spawn_loads(jobs, window.as_weak(), cache.clone());
}
