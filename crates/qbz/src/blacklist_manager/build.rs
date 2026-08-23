//! Build filtered view models from the store snapshots, and push them into
//! `BlacklistState`.

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{AppWindow, BlacklistState, BlacklistedArtistItem, DismissedArtistItem};

use super::build_album::build_album_items;
use super::state::{current_query, format_added, IMAGE_CACHE};

/// Build the visible (filtered) `BlacklistedArtistItem`s from the full
/// name-sorted snapshot, applying the current query.
fn build_items() -> (Vec<BlacklistedArtistItem>, i32) {
    let all = crate::artist_blacklist::get_all();
    let count = all.len() as i32;
    let query = current_query();
    let needle = query.trim().to_lowercase();

    let items: Vec<BlacklistedArtistItem> = all
        .into_iter()
        .filter(|a| needle.is_empty() || a.artist_name.to_lowercase().contains(&needle))
        .map(|a| {
            let notes = a.notes.clone().unwrap_or_default();
            BlacklistedArtistItem {
                // artist_id is u64; an int holds Qobuz ids comfortably for
                // display + passing back on click/remove (matches OfflineRow
                // carrying ids as strings — here the spec uses int).
                artist_id: a.artist_id as i32,
                artist_name: a.artist_name.into(),
                added_at: a.added_at as i32,
                added_display: format_added(a.added_at).into(),
                has_notes: !notes.is_empty(),
                notes: notes.into(),
            }
        })
        .collect();

    (items, count)
}

/// Build the visible (filtered) dismissed-artist items — the "Not interested"
/// reco-scoped list — from the store snapshot, applying the current query
/// (name match, same rule as the artist axis). `count` carries the FULL list
/// length for the tab badge + empty/no-results split.
fn build_dismissed_items() -> (Vec<DismissedArtistItem>, i32) {
    let all = crate::reco_dismiss::list();
    let count = all.len() as i32;
    let needle = current_query().trim().to_lowercase();

    let items: Vec<DismissedArtistItem> = all
        .into_iter()
        .filter(|a| needle.is_empty() || a.name.to_lowercase().contains(&needle))
        .map(|a| DismissedArtistItem {
            // Same int pass-through as the blacklist artist id.
            artist_id: a.artist_id as i32,
            artist_name: a.name.into(),
        })
        .collect();

    (items, count)
}

/// Push the filtered items + full count + enabled flag + query into Slint (all
/// three axes). Album covers resolve asynchronously via the shared image cache.
pub(super) fn push(w: &AppWindow) {
    let (items, count) = build_items();
    let (album_items, album_count, jobs) = build_album_items();
    let (dismissed_items, dismissed_count) = build_dismissed_items();
    let st = w.global::<BlacklistState>();
    st.set_items(ModelRc::new(VecModel::from(items)));
    st.set_count(count);
    st.set_album_items(ModelRc::new(VecModel::from(album_items)));
    st.set_album_count(album_count);
    st.set_dismissed_items(ModelRc::new(VecModel::from(dismissed_items)));
    st.set_dismissed_count(dismissed_count);
    st.set_enabled(crate::artist_blacklist::is_enabled());
    st.set_search_query(SharedString::from(current_query()));
    // Kick off cover loads (best-effort; needs the cache + a weak handle).
    if let Some(cache) = IMAGE_CACHE.get() {
        if !jobs.is_empty() {
            crate::artwork::spawn_loads(jobs, w.as_weak(), cache.clone());
        }
    }
}

/// Load (or refresh) the manager: mark loading, read the store, push state,
/// clear loading. Synchronous — the wrapper reads are in-memory / a single
/// SQLite query, so there is no worker hop (unlike the offline manager's
/// index scan).
pub fn load(weak: slint::Weak<AppWindow>) {
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<BlacklistState>().set_loading(true);
        push(&w);
        w.global::<BlacklistState>().set_loading(false);
    });
}
