//! The coverflow windowed lazy-decode step: emits a decode job only for a
//! window row whose model cell still lacks a decoded handle.

use slint::{ComponentHandle, Model};

use crate::queue::artwork::{load_artwork, ArtTarget};
use crate::{AppWindow, ImmersiveState, QueueState};

/// COVERFLOW windowed lazy decode (inside the event loop so it can read the
/// live model and SKIP rows that already carry a decoded handle). After a
/// rebuild the to_item_reuse map already filled most window covers; after an
/// index-only update the model is the prior one with handles intact. Either
/// way we emit a decode job ONLY for a window row whose model cell is still a
/// default (0-width) image -> at most ONE decode per advance (the cover that
/// just entered ±4), often zero. Visible covers are NEVER re-decoded (the
/// invariant).
///
/// The immersive QUEUE panel (focus mode==5 or split-panel==3, while
/// immersive is open) shows the WHOLE up-next as a list, so every row needs
/// art — widen the window to the full list there. The coverflow FAN (panel
/// closed) keeps the cheap ±4 window. Same gate shape as lyrics_sync's panel
/// detection.
pub(super) fn windowed_decode(
    w: &AppWindow,
    qs: &QueueState,
    cf_lo: usize,
    cf_hi: usize,
    coverflow_art_jobs: Vec<(usize, String)>,
) {
    let cf_model = qs.get_coverflow_tracks();
    let imm = w.global::<ImmersiveState>();
    let queue_panel_open = imm.get_open()
        && ((imm.get_view_mode() == 0 && imm.get_mode() == 5)
            || (imm.get_view_mode() == 1 && imm.get_split_panel() == 3));
    let mut windowed_jobs: Vec<(ArtTarget, String)> = Vec::new();
    for (flat_idx, url) in coverflow_art_jobs.into_iter() {
        let in_window = queue_panel_open || (flat_idx >= cf_lo && flat_idx <= cf_hi);
        if !in_window {
            continue;
        }
        let needs = cf_model
            .row_data(flat_idx)
            .map(|it| it.artwork.size().width == 0)
            .unwrap_or(false);
        if needs {
            windowed_jobs.push((ArtTarget::CoverflowFlat(flat_idx), url));
        }
    }
    if !windowed_jobs.is_empty() {
        load_artwork(w.as_weak(), windowed_jobs);
    }
}
