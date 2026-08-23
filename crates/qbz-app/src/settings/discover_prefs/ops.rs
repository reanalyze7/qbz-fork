use super::defaults::default_prefs;
use super::model::{DiscoverPrefs, SectionPref};
use super::section_id::DiscoverySectionId;
use super::tabs::DiscoveryTab;

impl DiscoverPrefs {
    pub fn tab(&self, tab: DiscoveryTab) -> &Vec<SectionPref> {
        match tab {
            DiscoveryTab::Home => &self.home,
            DiscoveryTab::EditorPicks => &self.editor_picks,
            DiscoveryTab::ForYou => &self.for_you,
        }
    }

    pub fn tab_mut(&mut self, tab: DiscoveryTab) -> &mut Vec<SectionPref> {
        match tab {
            DiscoveryTab::Home => &mut self.home,
            DiscoveryTab::EditorPicks => &mut self.editor_picks,
            DiscoveryTab::ForYou => &mut self.for_you,
        }
    }

    /// Flip `enabled` on the matching id. No minimum-enabled guard (can reach 0).
    pub fn toggle(&mut self, tab: DiscoveryTab, id: DiscoverySectionId) {
        if let Some(p) = self.tab_mut(tab).iter_mut().find(|p| p.id == id) {
            p.enabled = !p.enabled;
        }
    }

    /// Move a section one step (`dir` = -1 up / +1 down) with boundary clamp.
    /// No-op if the id is absent or already at the boundary. The `enabled`
    /// flag travels with the entry (the whole `SectionPref` is swapped).
    pub fn move_section(&mut self, tab: DiscoveryTab, id: DiscoverySectionId, dir: i8) {
        let list = self.tab_mut(tab);
        let Some(idx) = list.iter().position(|p| p.id == id) else {
            return;
        };
        if dir < 0 && idx > 0 {
            list.swap(idx, idx - 1);
        } else if dir > 0 && idx + 1 < list.len() {
            list.swap(idx, idx + 1);
        }
    }

    /// Replace one tab's list with a FRESH clone of its defaults.
    pub fn reset_tab(&mut self, tab: DiscoveryTab) {
        let defaults = default_prefs();
        *self.tab_mut(tab) = defaults.tab(tab).clone();
    }

    pub fn is_enabled(&self, tab: DiscoveryTab, id: DiscoverySectionId) -> bool {
        self.tab(tab)
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.enabled)
            .unwrap_or(false)
    }

    pub fn enabled_count(&self, tab: DiscoveryTab) -> usize {
        self.tab(tab).iter().filter(|p| p.enabled).count()
    }

    /// The ordered list of ENABLED section ids for a tab — drives the render
    /// loop in the frontend.
    pub fn enabled_ordered(&self, tab: DiscoveryTab) -> Vec<DiscoverySectionId> {
        self.tab(tab)
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.id)
            .collect()
    }

    /// The set of section ids a tab offers (= the ids in its DEFAULT order).
    /// The configurator only ever shows these for the tab.
    pub fn available_ids(tab: DiscoveryTab) -> Vec<DiscoverySectionId> {
        default_prefs().tab(tab).iter().map(|p| p.id).collect()
    }
}
