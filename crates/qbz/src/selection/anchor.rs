//! Per-surface Shift-range anchor (thread-local, UI thread only).

use std::cell::RefCell;

use slint::{Model, VecModel};

// ---------------------------------------------------------------------------
// Surface ids — a stable, arbitrary discriminator so an anchor set on one
// surface never leaks into a Shift-range on another. These are NOT 1:1 with
// `ContentView`: surfaces like LocalLibrary Tracks / Offline / a future albums
// grid select via their own controllers, not the central toggle arm.
// ---------------------------------------------------------------------------
pub const SURFACE_ALBUM: u16 = 1;
pub const SURFACE_ARTIST: u16 = 2;
pub const SURFACE_PLAYLIST: u16 = 3;
pub const SURFACE_FAVORITES: u16 = 4;
pub const SURFACE_LABEL: u16 = 5;
pub const SURFACE_LOCAL_TRACKS: u16 = 6;
pub const SURFACE_OFFLINE: u16 = 7;
pub const SURFACE_MIX: u16 = 8;
pub const SURFACE_LOCAL_ALBUMS: u16 = 9;

#[derive(Clone)]
struct Anchor {
    surface: u16,
    index: usize,
    /// The clicked row's id, kept so a Shift-range can re-resolve the anchor by
    /// id if the model was re-sorted/filtered under it (the index alone would
    /// go stale). Matches Tauri nulling `lastSelectedIndex` on model rebuild,
    /// but resilient without hooking every rebuild site.
    id: String,
}

thread_local! {
    static ANCHOR: RefCell<Option<Anchor>> = const { RefCell::new(None) };
}

/// Remember the clicked row as the new anchor for `surface`.
pub fn set_anchor(surface: u16, index: usize, id: &str) {
    ANCHOR.with(|a| {
        *a.borrow_mut() = Some(Anchor {
            surface,
            index,
            id: id.to_string(),
        });
    });
}

/// Drop the anchor (call on enter/leave select-mode and on model rebuild).
pub fn clear_anchor() {
    ANCHOR.with(|a| *a.borrow_mut() = None);
}

/// The stored anchor `(index, id)` for `surface`, if the current anchor belongs
/// to that surface. The caller verifies/re-resolves the index against the live
/// model before ranging.
pub fn anchor_for(surface: u16) -> Option<(usize, String)> {
    ANCHOR.with(|a| {
        a.borrow()
            .as_ref()
            .filter(|an| an.surface == surface)
            .map(|an| (an.index, an.id.clone()))
    })
}

/// Resolve the anchor index for `surface` against the live `model`, by id. Use
/// the stored index when the id at that index still matches; otherwise scan the
/// model for the stored id (it was re-sorted/filtered); `None` if it is gone.
/// `id_at` extracts a row's id (e.g. `|t| t.id.to_string()`).
pub fn resolve_anchor<T: Clone + 'static>(
    surface: u16,
    model: &VecModel<T>,
    id_at: impl Fn(&T) -> String,
) -> Option<usize> {
    let (idx, id) = anchor_for(surface)?;
    if let Some(item) = model.row_data(idx) {
        if id_at(&item) == id {
            return Some(idx);
        }
    }
    (0..model.row_count()).find(|&i| model.row_data(i).map(|t| id_at(&t) == id).unwrap_or(false))
}
