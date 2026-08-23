//! Toolbar state transitions: persist, filter/sort/search. The heavier
//! `refresh_view` re-derive lives in `refresh.rs`.

mod refresh;

use slint::ComponentHandle;

pub use refresh::refresh_view;

use crate::{AppWindow, MyQbzDetailState};

/// Persist the open collection's current toolbar prefs (spec 12 §18), gated on
/// the hydrated flag so a setter firing before `apply` restores the stored
/// prefs cannot clobber them. The persisted fields are the five §18 fields
/// (view-mode / sort / sort-dir / type-filter / source flags); search +
/// select-mode stay transient. UI thread.
pub fn persist_prefs(window: &AppWindow) {
    if !super::PREFS_HYDRATED.with(|c| c.get()) {
        return;
    }
    let state = window.global::<MyQbzDetailState>();
    let id = state.get_id().to_string();
    if id.is_empty() {
        return;
    }
    let prefs = crate::myqbz_view_prefs::Prefs {
        view_mode: state.get_view_mode().to_string(),
        sort_by: state.get_sort().to_string(),
        sort_dir: state.get_sort_dir().to_string(),
        type_filter: state.get_type_filter().to_string(),
        src_qobuz: state.get_src_qobuz(),
        src_local: state.get_src_local(),
    };
    crate::myqbz_view_prefs::save(&id, &prefs);
}

/// Update the search query and re-render.
pub fn search(window: &AppWindow, query: &str) {
    window.global::<MyQbzDetailState>().set_search(query.into());
    refresh_view(window);
}

/// Set the sort field. Re-selecting the active field flips asc/desc; a new
/// field resets to asc (spec 12 §5.4 `selectSort`).
pub fn set_sort(window: &AppWindow, field: &str) {
    let state = window.global::<MyQbzDetailState>();
    if state.get_sort() == field {
        let dir = if state.get_sort_dir() == "asc" { "desc" } else { "asc" };
        state.set_sort_dir(dir.into());
    } else {
        state.set_sort(field.into());
        state.set_sort_dir("asc".into());
    }
    persist_prefs(window);
    refresh_view(window);
}

/// Single-select the type filter.
pub fn set_type_filter(window: &AppWindow, value: &str) {
    window.global::<MyQbzDetailState>().set_type_filter(value.into());
    persist_prefs(window);
    refresh_view(window);
}

/// Toggle one source-filter flag (multi-select; menu stays open in the view).
pub fn toggle_source_filter(window: &AppWindow, kind: &str) {
    let state = window.global::<MyQbzDetailState>();
    match kind {
        "qobuz" => state.set_src_qobuz(!state.get_src_qobuz()),
        "local" => state.set_src_local(!state.get_src_local()),
        _ => {}
    }
    persist_prefs(window);
    refresh_view(window);
}

/// Reset filters + sort (spec 12 §5.6 reset: type 'all', no sources, sort
/// 'position' asc). Search query is left intact (Tauri's reset doesn't clear
/// it; `hasAnyFilter` excludes search).
pub fn reset_filters(window: &AppWindow) {
    let state = window.global::<MyQbzDetailState>();
    state.set_type_filter("all".into());
    state.set_src_qobuz(false);
    state.set_src_local(false);
    state.set_sort("position".into());
    state.set_sort_dir("asc".into());
    persist_prefs(window);
    refresh_view(window);
}

/// Set the view-mode (list|grid|expanded) + persist it (spec 12 §18). The
/// expanded-mode inline-track fetch stays in `main.rs` (it needs the runtime +
/// handle); this only updates state + persists so the per-collection prefs
/// remember the chosen mode.
pub fn set_view_mode(window: &AppWindow, mode: &str) {
    window.global::<MyQbzDetailState>().set_view_mode(mode.into());
    persist_prefs(window);
}
