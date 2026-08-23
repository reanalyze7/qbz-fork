//! Applying one resolved cover onto its queue/coverflow row.

use slint::{ComponentHandle, Model};

use super::ArtTarget;
use crate::{AppWindow, QueueState};

/// Apply a resolved cover onto its queue/coverflow row. Runs on the event loop.
pub(in crate::queue) fn apply_queue_art(win: &AppWindow, target: ArtTarget, img: slint::Image) {
    let qs = win.global::<QueueState>();
    match target {
        ArtTarget::NowPlaying => {
            let mut item = qs.get_now_playing();
            item.artwork = img;
            qs.set_now_playing(item);
        }
        ArtTarget::Upcoming(idx) => {
            let items = qs.get_upcoming_page();
            if let Some(mut item) = items.row_data(idx) {
                item.artwork = img;
                items.set_row_data(idx, item);
            }
        }
        ArtTarget::History(idx) => {
            let items = qs.get_history();
            if let Some(mut item) = items.row_data(idx) {
                item.artwork = img;
                items.set_row_data(idx, item);
            }
        }
        ArtTarget::CoverflowFlat(idx) => {
            let items = qs.get_coverflow_tracks();
            if let Some(mut item) = items.row_data(idx) {
                item.artwork = img.clone();
                // set_row_data on a SINGLE flat row — does NOT replace the
                // VecModel, so the Repeater is not rebuilt and the other covers'
                // sources are untouched. Only this one cell's image changes.
                items.set_row_data(idx, item);
            }
            // Mirror the decode onto the reversed model used by the RIGHT
            // Repeater. The reversed model is the forward list reversed, so the
            // same track sits at `len-1-idx`. Updating a single row here (not
            // replacing the VecModel) keeps that Repeater stable too — no rebuild,
            // no churn — it just fills in the same cover the forward side got.
            let rev = qs.get_coverflow_tracks_rev();
            let rev_len = rev.row_count();
            if rev_len > 0 && idx < rev_len {
                let rev_idx = rev_len - 1 - idx;
                if let Some(mut item) = rev.row_data(rev_idx) {
                    item.artwork = img;
                    rev.set_row_data(rev_idx, item);
                }
            }
        }
    }
}
