//! The cortinilla / immersive-search-row arms — split out of
//! `search_immersive.rs` only for line budget; both share the same
//! URL-match late-arrival-guard idiom (a slow load from a previous query
//! must not paint the wrong cover onto a flat-index a new query reused).

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::{AppWindow, SearchState};

pub(in crate::artwork) fn apply(
    window: &AppWindow,
    target: ArtworkTarget,
    url: &str,
    image: &slint::Image,
) -> bool {
    match target {
        ArtworkTarget::CortinillaRow { flat_index } => {
            let state = window.global::<SearchState>();
            // Late-arrival guard (URL match): only paint if the row STILL carries
            // the exact URL we loaded. The cortinilla re-renders on every
            // keystroke and REUSES flat indices, so a slow load from a previous
            // query would otherwise paint the wrong cover onto the new row that
            // now occupies the same flat-index — the "momentary wrong cover" the
            // image cache is too slow to avoid. Matching the URL makes a stale
            // load a true no-op.
            if flat_index == 0 {
                let mut top = state.get_top_result();
                if top.artwork_url.as_str() == url {
                    top.artwork = image.clone();
                    state.set_top_result(top);
                }
            } else {
                let sections = state.get_sections();
                'outer: for s in 0..sections.row_count() {
                    if let Some(section) = sections.row_data(s) {
                        let rows = section.rows.clone();
                        for r in 0..rows.row_count() {
                            if let Some(mut row) = rows.row_data(r) {
                                if row.flat_index as usize == flat_index
                                    && row.artwork_url.as_str() == url
                                {
                                    row.artwork = image.clone();
                                    rows.set_row_data(r, row);
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }
        }
        ArtworkTarget::ImmersiveSearchRow { flat_index } => {
            // Same late-arrival URL-match guard as `CortinillaRow`: only paint
            // when the row STILL carries the exact URL we loaded. The immersive
            // cortinilla re-renders on every keystroke and REUSES flat indices,
            // so a slow load from a previous query would otherwise paint the
            // wrong cover onto the new row that now occupies the same flat-index.
            // The immersive cortinilla has no top result, so flat-index 0 is
            // never produced — every job is a section row.
            let sections = window.global::<crate::ImmersiveState>().get_search_sections();
            'outer: for s in 0..sections.row_count() {
                if let Some(section) = sections.row_data(s) {
                    let rows = section.rows.clone();
                    for r in 0..rows.row_count() {
                        if let Some(mut row) = rows.row_data(r) {
                            if row.flat_index as usize == flat_index
                                && row.artwork_url.as_str() == url
                            {
                                row.artwork = image.clone();
                                rows.set_row_data(r, row);
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        _ => return false,
    }
    true
}
