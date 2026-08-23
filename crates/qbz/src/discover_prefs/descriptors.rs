//! Descriptor builders: map `DiscoverPrefs` into the Slint `DiscoverState` /
//! `SettingsState` / `ExternalRecoState` globals.

use qbz_app::settings::discover_prefs::{DiscoverPrefs, DiscoverySectionId, DiscoveryTab};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use super::labels::label_for;
use super::reco_ttl::ttl_index_from_hours;
use super::{current, render_kind};
use crate::{
    AppWindow, ConfigRow, DiscoverSection, DiscoverState, ExternalRecoState, SectionDescriptor,
    SettingsState,
};

/// A descriptor with no embedded album payload (For You arms dispatch on id and
/// read the typed ForYouState fields). `section` is an empty default.
fn bare_descriptor(id: DiscoverySectionId) -> SectionDescriptor {
    SectionDescriptor {
        id: SharedString::from(id.as_str()),
        kind: SharedString::from(render_kind(id)),
        section: DiscoverSection::default(),
    }
}

/// For You ordered ENABLED descriptors. The For You delegate reads ForYouState
/// by id and keeps its own `length > 0` self-hide gate (qobuzMixes excepted), so
/// the descriptor list is the pure visibility+order driver and carries no album
/// payload. `essentialsByGenre` is DROPPED here: it is Slice-2c-blocked (no
/// `ForYouState.essentials` field exists yet), so emitting it would mount a
/// delegate with no matching arm. It re-appears automatically once Slice 2c adds
/// the field and an arm.
fn foryou_descriptors(prefs: &DiscoverPrefs) -> Vec<SectionDescriptor> {
    prefs
        .enabled_ordered(DiscoveryTab::ForYou)
        .into_iter()
        .filter(|id| *id != DiscoverySectionId::EssentialsByGenre)
        .map(bare_descriptor)
        .collect()
}

/// Push the descriptor lists for ALL three tabs. Home / Editor's Picks lists are
/// built by `crate::home` from the cached section data (the album-carousel arms
/// embed it); For You is built here. When the active tab is For You the Home /
/// Editor lists are pushed EMPTY so the Home repeater renders nothing for that
/// tab — content is controlled purely via the model, avoiding a conditional
/// repeater (preferred unconditional-repeater form; see HomeView).
pub fn push_descriptors(window: &AppWindow, prefs: &DiscoverPrefs) {
    let state = window.global::<DiscoverState>();
    let active = state.get_active_tab().to_string();

    // For You list (always pushed; the For You view is mounted only when active).
    state.set_foryou_sections(ModelRc::new(VecModel::from(foryou_descriptors(prefs))));

    if active == "forYou" || active == "recommendations" {
        // Drive the Home repeater empty for the For You + Recommendations tabs
        // (both render from their own dedicated views).
        state.set_home_sections(ModelRc::new(VecModel::from(Vec::<SectionDescriptor>::new())));
        state.set_editor_sections(ModelRc::new(VecModel::from(Vec::<SectionDescriptor>::new())));
    } else {
        // Home + Editor descriptors come from the cached section data.
        let (home, editor) = crate::home::tab_descriptors(prefs);
        state.set_home_sections(ModelRc::new(VecModel::from(home)));
        state.set_editor_sections(ModelRc::new(VecModel::from(editor)));
    }
}

/// Push the configurator modal payload for one tab: the FULL ordered list
/// (enabled AND disabled), with labels resolved in Rust, plus the enabled/total
/// counts. `can-move-up/down` are NOT struct fields — the modal computes boundary
/// state from the row index, so the struct stays minimal.
pub fn push_config_rows(window: &AppWindow, prefs: &DiscoverPrefs, tab: DiscoveryTab) {
    let rows: Vec<ConfigRow> = prefs
        .tab(tab)
        .iter()
        .map(|p| ConfigRow {
            id: SharedString::from(p.id.as_str()),
            label: SharedString::from(qbz_i18n::t(label_for(p.id))),
            enabled: p.enabled,
        })
        .collect();
    let total = rows.len() as i32;
    let enabled = prefs.enabled_count(tab) as i32;
    let state = window.global::<DiscoverState>();
    state.set_config_rows(ModelRc::new(VecModel::from(rows)));
    state.set_enabled_count(enabled);
    state.set_total_count(total);
}

/// Seed the descriptor lists at shell entry so the render loop has data before
/// the first `apply_home`. Mirrors `myqbz_prefs::seed`.
pub fn seed(window: &AppWindow) {
    let prefs = current();
    window
        .global::<SettingsState>()
        .set_show_recommendations(prefs.show_recommendations);
    // Seed the Recommendations cache-window select to the persisted choice.
    window
        .global::<ExternalRecoState>()
        .set_cache_ttl_index(ttl_index_from_hours(prefs.reco_cache_ttl_hours));
    // MusicBrainz opt-out lives in ui_prefs (not the discover prefs store).
    // Seed it here so both SettingsState seed sites (main.rs:320/554) populate
    // the toggle that gates the artist sidebar + playlist suggestions.
    window
        .global::<SettingsState>()
        .set_musicbrainz_enabled(crate::ui_prefs::load().musicbrainz_enabled);
    push_descriptors(window, &prefs);
}
