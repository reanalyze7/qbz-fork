//! `refresh_view` — apply the active toolbar over `FULL_ITEMS` and push the
//! resulting render model.

use qbz_models::mixtape::{ItemType, MixtapeCollectionItem};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::myqbz_detail::model::to_item;
use crate::myqbz_detail::strings::{item_type_str, source_str};
use crate::myqbz_detail::FULL_ITEMS;
use crate::{AppWindow, MixtapeDetailItem, MyQbzDetailState};

/// Apply the active toolbar (type filter -> source filter -> search -> sort)
/// over `FULL_ITEMS` and push the resulting render model. Non-destructive (the
/// persisted order is untouched). UI thread only. Mirrors spec 12 §19.
pub fn refresh_view(window: &AppWindow) {
    let state = window.global::<MyQbzDetailState>();
    let query = state.get_search().trim().to_lowercase();
    let type_filter = state.get_type_filter().to_string();
    let (sq, sl) = (state.get_src_qobuz(), state.get_src_local());
    let any_source = sq || sl;
    let sort = state.get_sort().to_string();
    let desc = state.get_sort_dir().to_string() == "desc";

    let mut view: Vec<MixtapeCollectionItem> = FULL_ITEMS.with(|cell| {
        cell.borrow()
            .iter()
            .filter(|it| {
                // Drop blocked albums by their own id. Album-type Qobuz items
                // only — `source_item_id` is the Qobuz album id.
                !(matches!(it.item_type, ItemType::Album)
                    && source_str(it.source) == "qobuz"
                    && crate::artist_blacklist::is_album_blacklisted(&it.source_item_id))
            })
            .filter(|it| {
                // Type filter (single-select).
                type_filter == "all" || item_type_str(it.item_type) == type_filter
            })
            .filter(|it| {
                // Source filter (multi-select). source_kind currently equals
                // the raw source (resolveItems deferred) — qobuz / local.
                if !any_source {
                    return true;
                }
                let kind = source_str(it.source);
                (sq && kind == "qobuz") || (sl && kind == "local")
            })
            .filter(|it| {
                if query.is_empty() {
                    return true;
                }
                it.title.to_lowercase().contains(&query)
                    || it
                        .subtitle
                        .as_deref()
                        .map(|s| s.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    });

    match sort.as_str() {
        "name" => view.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
        "year" => view.sort_by(|a, b| a.year.unwrap_or(0).cmp(&b.year.unwrap_or(0))),
        "tracks" => {
            view.sort_by(|a, b| a.track_count.unwrap_or(0).cmp(&b.track_count.unwrap_or(0)))
        }
        // default "position"
        _ => view.sort_by(|a, b| a.position.cmp(&b.position)),
    }
    if desc {
        view.reverse();
    }

    let items: Vec<MixtapeDetailItem> = view.iter().map(to_item).collect();
    state.set_items(ModelRc::new(VecModel::from(items)));

    // Derived toolbar badges (Rust-computed; the view only reads them).
    let source_count = (sq as i32) + (sl as i32);
    state.set_filter_count(source_count + if type_filter != "all" { 1 } else { 0 });
    state.set_has_any_filter(
        type_filter != "all" || any_source || sort != "position" || desc,
    );
}
