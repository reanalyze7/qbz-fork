//! DB read, and the reset/apply/not-found lifecycle steps.

use qbz_models::mixtape::MixtapeCollection;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::myqbz_detail::hero::apply_hero_mosaic;
use crate::myqbz_detail::strings::{album_count_label, kind_label, kind_str, play_mode_str};
use crate::myqbz_detail::toolbar::refresh_view;
use crate::myqbz_detail::{FULL_ITEMS, INLINE_CACHE, PREFS_HYDRATED, RESOLVE_CACHE};
use crate::{AppWindow, MixtapeDetailItem, MyQbzDetailState};

/// Load one collection (items hydrated by the repo) from the per-user
/// library.db. Returns `None` when the DB is unavailable or the id is unknown.
pub fn get_collection(id: &str) -> Option<MixtapeCollection> {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            qbz_mixtape::repo::get_collection(conn, id).unwrap_or_else(|e| {
                log::warn!("[qbz-slint] myqbz_detail get_collection({id}) failed: {e}");
                None
            })
        }))
    })
    .flatten()
}

/// Clear the view to its loading state before a fresh load (so a re-open does
/// not flash the previous collection's hero + rows).
pub fn reset(window: &AppWindow) {
    FULL_ITEMS.with(|cell| cell.borrow_mut().clear());
    // Drop the inline-tracks cache — a different collection's tracks must not
    // leak into the freshly-opened one.
    INLINE_CACHE.with(|cell| cell.borrow_mut().clear());
    // Drop the resolveItems cache too (same reason — the resolved source/
    // quality/type of a different collection's items must not leak).
    RESOLVE_CACHE.with(|cell| cell.borrow_mut().clear());
    // Close the persist gate until `apply` restores this collection's prefs —
    // any toolbar setter that fires meanwhile must NOT overwrite stored prefs
    // with the in-flight defaults (mirrors Tauri's prefsHydrated).
    PREFS_HYDRATED.with(|c| c.set(false));
    let state = window.global::<MyQbzDetailState>();
    state.set_loading(true);
    state.set_found(true);
    state.set_items(ModelRc::new(VecModel::from(Vec::<MixtapeDetailItem>::new())));
    state.set_name("".into());
    state.set_description("".into());
    state.set_meta("".into());
    state.set_item_count(0);
    state.set_has_custom_cover(false);
    state.set_custom_cover(slint::Image::default());
    state.set_cover_count(0);
    state.set_selected_count(0);
    state.set_select_mode(false);
    // Toolbar -> defaults during load; `apply` then restores this collection's
    // persisted view-prefs (spec 12 §18) over these. Search + select-mode stay
    // transient (never persisted) so they always start fresh.
    state.set_search("".into());
    state.set_sort("position".into());
    state.set_sort_dir("asc".into());
    state.set_type_filter("all".into());
    state.set_src_qobuz(false);
    state.set_src_local(false);
    state.set_view_mode("list".into());
    state.set_filter_count(0);
    state.set_has_any_filter(false);
}

/// Apply a freshly-loaded collection: header strings, hero mosaic, the full
/// item list (-> FULL_ITEMS), then render through the (reset) toolbar.
pub fn apply(window: &AppWindow, c: MixtapeCollection) {
    let state = window.global::<MyQbzDetailState>();
    let item_count = c.items.len();

    state.set_id(c.id.clone().into());
    state.set_kind(kind_str(c.kind).into());
    state.set_kind_label(kind_label(c.kind).into());
    state.set_name(c.name.clone().into());
    state.set_description(c.description.clone().unwrap_or_default().into());
    state.set_meta(album_count_label(item_count).into());
    state.set_item_count(item_count as i32);
    state.set_play_mode(play_mode_str(c.play_mode).into());
    state.set_found(true);

    // Custom cover (overrides the mosaic) — load the local file directly (it
    // lives in the artwork cache on disk; same as the playlist controller),
    // decoded to the card tier (the hero renders it at 186px).
    let has_custom = c
        .custom_artwork_path
        .as_ref()
        .filter(|p| !p.is_empty())
        .filter(|p| std::path::Path::new(p).exists())
        .and_then(|p| crate::artwork::load_local_cover(p, 264));
    if let Some(img) = has_custom {
        state.set_has_custom_cover(true);
        state.set_custom_cover(img);
    } else {
        state.set_has_custom_cover(false);
        state.set_custom_cover(slint::Image::default());
    }

    apply_hero_mosaic(&state, &c);

    // Restore this collection's persisted view-prefs over the reset defaults
    // (spec 12 §18). `load` returns the §18 defaults when nothing is stored, so
    // a never-opened collection lands on list/position/asc/all/empty exactly as
    // before. Open the persist gate AFTER applying so the restore itself isn't
    // re-persisted (and so subsequent setter-driven persists are live).
    let prefs = crate::myqbz_view_prefs::load(c.id.as_str());
    state.set_view_mode(prefs.view_mode.into());
    state.set_sort(prefs.sort_by.into());
    state.set_sort_dir(prefs.sort_dir.into());
    state.set_type_filter(prefs.type_filter.into());
    state.set_src_qobuz(prefs.src_qobuz);
    state.set_src_local(prefs.src_local);
    PREFS_HYDRATED.with(|cell| cell.set(true));

    FULL_ITEMS.with(|cell| *cell.borrow_mut() = c.items);
    refresh_view(window);
    state.set_loading(false);
}

/// Mark the load as not-found (the id resolved to no collection).
pub fn apply_not_found(window: &AppWindow) {
    let state = window.global::<MyQbzDetailState>();
    state.set_loading(false);
    state.set_found(false);
}
