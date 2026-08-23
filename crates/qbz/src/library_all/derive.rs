//! Sort toggle + the search/filter/sort derivation into `items-visible`.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, LibraryAllState, LibraryFeedItem};

/// PlaylistView-style sort toggle: re-selecting the active field flips its
/// direction; a new field resets to that field's natural default ("date"
/// newest-first, "title"/"artist" A→Z). Then re-derive.
pub fn set_sort(window: &AppWindow, field: &str) {
    let st = window.global::<LibraryAllState>();
    let cur_field = st.get_sort_by().to_string();
    let new_asc = if cur_field == field {
        !st.get_sort_asc()
    } else {
        // "date" starts descending (newest first); the others start ascending.
        field != "date"
    };
    st.set_sort_by(field.into());
    st.set_sort_asc(new_asc);
    derive(window);
}

/// Apply search + source-switch + genre + sort over the full model into
/// `items-visible`. Runs on the Slint event loop; Slint never sorts/filters.
pub fn derive(window: &AppWindow) {
    let st = window.global::<LibraryAllState>();
    let needle = st.get_search().to_lowercase();
    let show_purchases = st.get_show_purchases();
    let show_favorites = st.get_show_favorites();
    let show_following = st.get_show_following();
    let show_local = st.get_show_local();
    let sort_by = st.get_sort_by();
    let sort_asc = st.get_sort_asc();
    // Shared genre filter (its own "library-all" context). Empty = no filter;
    // otherwise an item shows only when its (lowercased) genre matches one of
    // the selected genre names — kinds with no genre (artist/label/playlist)
    // are excluded, so the feed narrows to the chosen genre's albums + tracks.
    let genre_names: Vec<String> = crate::genre_filter::selected_names("library-all")
        .into_iter()
        .map(|g| g.to_lowercase())
        .collect();

    let full = st.get_items();
    let mut out: Vec<LibraryFeedItem> = Vec::new();
    for i in 0..full.row_count() {
        let Some(item) = full.row_data(i) else {
            continue;
        };
        let src = item.source.as_str();
        let is_local = src == "local";
        if is_local {
            // Local files are gated ONLY by the show-local switch; they
            // bypass the Qobuz purchases/favorites/following switches.
            if !show_local {
                continue;
            }
        } else {
            // Qobuz source switches: an item shows when its group's switch is on.
            // If ALL three are off, treat as "no filter" (show everything) to
            // avoid an empty grid from an accidental all-off state.
            let any_group = show_purchases || show_favorites || show_following;
            let group = item.group.as_str();
            let group_ok = !any_group
                || (group == "purchases" && show_purchases)
                || (group == "favorites" && show_favorites)
                || (group == "following" && show_following);
            if !group_ok {
                continue;
            }
        }
        if !needle.is_empty() {
            let hit = item.sort_title.as_str().contains(&needle)
                || item.sort_artist.as_str().contains(&needle);
            if !hit {
                continue;
            }
        }
        if !genre_names.is_empty() {
            let g = item.genre.as_str();
            if g.is_empty() || !genre_names.iter().any(|n| g.contains(n.as_str())) {
                continue;
            }
        }
        out.push(item);
    }

    // Canonical ascending order per field, then reverse for the other
    // direction. "date" has no key on the item (the model is stored
    // newest-first from load), so it uses the inherent order: asc(false) =
    // newest-first (default), asc(true) = oldest-first (reversed).
    match sort_by.as_str() {
        "title" => {
            out.sort_by(|a, b| a.sort_title.as_str().cmp(b.sort_title.as_str()));
            if !sort_asc {
                out.reverse();
            }
        }
        "artist" => {
            out.sort_by(|a, b| a.sort_artist.as_str().cmp(b.sort_artist.as_str()));
            if !sort_asc {
                out.reverse();
            }
        }
        // "date": model order is newest-first; reverse only for oldest-first.
        _ => {
            if sort_asc {
                out.reverse();
            }
        }
    }

    st.set_items_visible(ModelRc::new(VecModel::from(out)));
}
