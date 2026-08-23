//! Public read/mutate API over the dismiss store.

use std::collections::HashSet;

use super::io::{load_store, write_store};
use super::DismissedArtist;

/// Snapshot of the dismissed id set — the §B paint-filter input. Empty when
/// no session is bound. Id 0 (a corrupt row) never matches.
pub fn ids_snapshot() -> HashSet<u64> {
    load_store()
        .artists
        .into_iter()
        .map(|a| a.artist_id)
        .filter(|id| *id != 0)
        .collect()
}

/// All dismissed artists in insertion order, for the manager tab. Empty on no
/// session or a corrupt file.
pub fn list() -> Vec<DismissedArtist> {
    load_store()
        .artists
        .into_iter()
        .filter(|a| a.artist_id != 0)
        .collect()
}

/// Record a dismissal (idempotent). A re-dismiss with a richer snapshot
/// backfills a previously empty name/image (e.g. first dismissed offline,
/// where the name could not be resolved).
pub fn dismiss(artist_id: u64, name: &str, image_url: &str) {
    if artist_id == 0 {
        return;
    }
    let mut store = load_store();
    match store.artists.iter().position(|a| a.artist_id == artist_id) {
        Some(idx) => {
            let existing = &mut store.artists[idx];
            if existing.name.is_empty() && !name.is_empty() {
                existing.name = name.to_string();
            }
            if existing.image_url.is_empty() && !image_url.is_empty() {
                existing.image_url = image_url.to_string();
            }
        }
        None => store.artists.push(DismissedArtist {
            artist_id,
            name: name.to_string(),
            image_url: image_url.to_string(),
        }),
    }
    write_store(&store);
}

/// Remove a dismissal (the manager tab's undo). No-op when absent / unbound.
pub fn remove(artist_id: u64) {
    let mut store = load_store();
    let before = store.artists.len();
    store.artists.retain(|a| a.artist_id != artist_id);
    if store.artists.len() != before {
        write_store(&store);
    }
}
