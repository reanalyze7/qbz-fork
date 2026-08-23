//! The body that runs INSIDE the event loop: snapshots prior-artwork
//! handles, writes every model onto `QueueState`, and (gated on
//! `seq_changed`) rebuilds the coverflow flat model.

use slint::ComponentHandle;

use super::prior_artwork::snapshot_prior_artwork;
use super::windowed_decode::windowed_decode;
use super::super::coverflow::CoverflowRows;
use super::super::snapshot::SnapshotRows;
use crate::queue::artwork::to_item_reuse;
use crate::{QueueItem, QueueState};

pub(super) fn push_in_event_loop(
    w: &crate::AppWindow,
    rows: SnapshotRows,
    cf: CoverflowRows,
    infinite: bool,
    cf_lo: usize,
    cf_hi: usize,
    coverflow_art_jobs: Vec<(usize, String)>,
) {
    let qs = w.global::<QueueState>();
    // Reflect the core's "stop after this song" marker so the .slint can
    // render the CircleStop on the matching row (item.id == stop-after-id).
    qs.set_stop_after_id(rows.stop_after_id);

    let prior_all = snapshot_prior_artwork(&qs);

    let np_item = rows
        .now_playing
        .as_ref()
        .map(|r| to_item_reuse(r, &prior_all))
        .unwrap_or_default();
    qs.set_has_current(rows.now_playing.is_some());
    qs.set_now_playing(np_item);
    qs.set_now_playing_favorite(rows.now_playing_favorite);

    let page_items: Vec<QueueItem> =
        rows.page_rows.iter().map(|r| to_item_reuse(r, &prior_all)).collect();
    qs.set_upcoming_page(slint::ModelRc::new(slint::VecModel::from(page_items)));
    qs.set_upcoming_total(rows.upcoming_total as i32);
    qs.set_upcoming_remaining(rows.remaining as i32);
    qs.set_page(rows.page as i32);
    qs.set_page_count(rows.page_count as i32);
    qs.set_page_start(rows.page_start as i32);
    qs.set_page_end(rows.page_end as i32);

    let history_items: Vec<QueueItem> =
        rows.history_rows.iter().map(|r| to_item_reuse(r, &prior_all)).collect();
    qs.set_history(slint::ModelRc::new(slint::VecModel::from(history_items)));

    // --- COVERFLOW: gated flat-model update -----------------------
    // KEY INVARIANT. On a PURE ADVANCE/JUMP the id-sequence is unchanged
    // (`!seq_changed`) so we DO NOT call set_coverflow_tracks: the Repeater
    // model is untouched, every visible cover keeps its decoded
    // `slint::Image` handle (no source reassignment, no re-decode), and only
    // the int `coverflow-index` moves -> the .slint `scroll` float animates
    // the fan to the new position. The model is rebuilt ONLY when the
    // contents actually change (new queue / shuffle / add / remove), reusing
    // the global id->handle map so even then only the genuinely-new covers
    // decode.
    if cf.seq_changed {
        let cf_items: Vec<QueueItem> =
            cf.rows.iter().map(|r| to_item_reuse(r, &prior_all)).collect();
        // The reversed model: same QueueItems (so a track's element holds the
        // SAME decoded handle on both sides — no extra decode), just in
        // reverse order. The RIGHT Repeater iterates THIS so it paints
        // far-upcoming -> near-upcoming, putting the nearer cover on top.
        // Rebuilt under the SAME seq gate as the forward model, so a pure
        // advance never touches it either.
        let mut cf_items_rev = cf_items.clone();
        cf_items_rev.reverse();
        qs.set_coverflow_tracks(slint::ModelRc::new(slint::VecModel::from(cf_items)));
        qs.set_coverflow_tracks_rev(slint::ModelRc::new(slint::VecModel::from(cf_items_rev)));
        qs.set_coverflow_seq_hash(cf.seq_hash as i32);
        log::debug!(
            "[coverflow-perf] rebuild seq={} len={} idx={}",
            cf.seq_hash,
            cf.rows.len(),
            cf.index
        );
    } else {
        log::debug!("[coverflow-perf] index-only idx={} (seq unchanged)", cf.index);
    }
    qs.set_coverflow_index(cf.index as i32);

    qs.set_infinite_play(infinite);
    // Keep the Slint tab property in sync with the view state so the
    // Queue/History body always matches the selected tab.
    qs.set_tab(rows.tab);

    windowed_decode(w, &qs, cf_lo, cf_hi, coverflow_art_jobs);
}
