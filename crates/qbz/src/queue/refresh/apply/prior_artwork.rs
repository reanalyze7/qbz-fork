//! Snapshotting prior decoded artwork handles before the models are
//! replaced (the CPU-spike / flicker fix's data source).

use slint::Model;

use crate::QueueState;

/// Snapshot prior decoded handles into ONE GLOBAL id -> artwork map covering
/// EVERY prior list (now-playing + upcoming + history + both coverflow
/// lists) BEFORE replacing the models. Coverflow navigation shifts a cover
/// ACROSS lists every click (now-playing -> history, upcoming -> now-playing,
/// ...), so a per-list diff misses the moved covers -> they blank to default
/// -> full re-decode -> flicker + CPU spike. A global map reuses a cover's
/// decoded handle no matter which list it sat in before; net per click only
/// the one genuinely new track decodes.
pub(super) fn snapshot_prior_artwork(
    qs: &QueueState,
) -> std::collections::HashMap<slint::SharedString, slint::Image> {
    let mut prior_all: std::collections::HashMap<slint::SharedString, slint::Image> =
        std::collections::HashMap::new();
    let np = qs.get_now_playing();
    if np.artwork.size().width > 0 {
        prior_all.insert(np.id.clone(), np.artwork.clone());
    }
    for m in [qs.get_upcoming_page(), qs.get_history(), qs.get_coverflow_tracks()] {
        for i in 0..m.row_count() {
            if let Some(it) = m.row_data(i) {
                if it.artwork.size().width > 0 {
                    prior_all.entry(it.id.clone()).or_insert(it.artwork.clone());
                }
            }
        }
    }
    prior_all
}
