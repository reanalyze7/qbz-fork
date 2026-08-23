//! Cortinilla (live search dropdown) row shaping: mapping combined-search
//! results into the labelled-section dropdown payload, for both the main
//! header cortinilla and the in-immersive variant.

mod immersive;
mod main;

pub use immersive::map_search_all_to_immersive;
pub use main::map_search_all_to_cortinilla;

use super::rows::CortinillaData;

/// (Re)assign `flat_index` across a cortinilla payload: top-result = 0, then
/// each section's rows in declaration order, 1..N. Called after the local
/// section is appended too, so indices stay contiguous.
pub fn assign_flat_indices(data: &mut CortinillaData) {
    let mut next = 0usize;
    if let Some(top) = data.top.as_mut() {
        top.flat_index = next;
    }
    // Whether or not there is a top result, section rows start at 1 (index 0 is
    // reserved for the top-result slot the overlay always treats as flat 0).
    next = 1;
    for section in data.sections.iter_mut() {
        for row in section.rows.iter_mut() {
            row.flat_index = next;
            next += 1;
        }
    }
}
