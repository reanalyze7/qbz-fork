//! Optimistic local mutations: cache-first flag flips / folder moves /
//! custom-order reorders, each followed by a `rebuild`.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, PlaylistManagerState};

use super::render::rebuild;
use super::types::CACHE;

/// Flip a playlist's favorite flag in the cache + rebuild.
pub fn toggle_favorite_local(window: &AppWindow, playlist_id: u64) -> bool {
    let mut new_val = false;
    if let Ok(mut c) = CACHE.lock() {
        if let Some(p) = c.playlists.iter_mut().find(|p| p.id == playlist_id) {
            p.is_favorite = !p.is_favorite;
            new_val = p.is_favorite;
        }
    }
    rebuild(window);
    new_val
}

/// Flip a playlist's hidden flag in the cache + rebuild.
pub fn toggle_hidden_local(window: &AppWindow, playlist_id: u64) -> bool {
    let mut new_val = false;
    if let Ok(mut c) = CACHE.lock() {
        if let Some(p) = c.playlists.iter_mut().find(|p| p.id == playlist_id) {
            p.is_hidden = !p.is_hidden;
            new_val = p.is_hidden;
        }
    }
    rebuild(window);
    new_val
}

/// Flip a LOCAL playlist's favorite flag in the cache + rebuild (B3).
/// Returns the new value for the repo write.
pub fn toggle_local_favorite(window: &AppWindow, id: &str) -> bool {
    let mut new_val = false;
    if let Ok(mut c) = CACHE.lock() {
        if let Some(p) = c.locals.iter_mut().find(|p| p.id == id) {
            p.is_favorite = !p.is_favorite;
            new_val = p.is_favorite;
        }
    }
    rebuild(window);
    new_val
}

/// Flip a LOCAL playlist's hidden flag in the cache + rebuild (B3).
/// Returns the new value for the repo write.
pub fn toggle_local_hidden(window: &AppWindow, id: &str) -> bool {
    let mut new_val = false;
    if let Ok(mut c) = CACHE.lock() {
        if let Some(p) = c.locals.iter_mut().find(|p| p.id == id) {
            p.is_hidden = !p.is_hidden;
            new_val = p.is_hidden;
        }
    }
    rebuild(window);
    new_val
}

/// Move a playlist into a folder ("" = root) in the cache + rebuild.
pub fn move_to_folder_local(window: &AppWindow, playlist_id: u64, folder_id: &str) {
    if let Ok(mut c) = CACHE.lock() {
        if let Some(p) = c.playlists.iter_mut().find(|p| p.id == playlist_id) {
            p.folder_id = if folder_id.is_empty() {
                None
            } else {
                Some(folder_id.to_string())
            };
        }
    }
    rebuild(window);
}

/// Move a playlist one slot up (custom sort): swap with its predecessor in
/// the current visible order, write the new positions back to the cache,
/// rebuild, and return the new full id order for persistence (empty when
/// the move is a no-op, e.g. already first).
pub fn move_up(window: &AppWindow, playlist_id: u64) -> Vec<u64> {
    reorder_step(window, playlist_id, -1)
}

/// Move a playlist one slot down (custom sort).
pub fn move_down(window: &AppWindow, playlist_id: u64) -> Vec<u64> {
    reorder_step(window, playlist_id, 1)
}

/// Shared up/down logic: reorder the currently-visible list (root, custom
/// sort) and assign fresh positions to the *full* playlist set so the
/// `position` field stays a total order. Returns the new id order for the
/// DB write, or empty on a no-op.
fn reorder_step(window: &AppWindow, playlist_id: u64, delta: i32) -> Vec<u64> {
    let model = window.global::<PlaylistManagerState>().get_playlists();
    let mut ids: Vec<u64> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter_map(|it| it.id.parse::<u64>().ok())
        .collect();
    let Some(pos) = ids.iter().position(|&id| id == playlist_id) else {
        return Vec::new();
    };
    let target = pos as i32 + delta;
    if target < 0 || target as usize >= ids.len() {
        return Vec::new();
    }
    ids.swap(pos, target as usize);

    // Write fresh positions back into the cache for the reordered ids, then
    // rebuild so the move is reflected immediately under the custom sort.
    if let Ok(mut c) = CACHE.lock() {
        for (i, id) in ids.iter().enumerate() {
            if let Some(p) = c.playlists.iter_mut().find(|p| p.id == *id) {
                p.position = i as i32;
            }
        }
    }
    rebuild(window);
    ids
}
