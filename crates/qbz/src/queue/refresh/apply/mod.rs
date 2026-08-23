//! The event-loop push step: builds the artwork-job lists, then hands off
//! to `push.rs`'s `upgrade_in_event_loop` closure body.

mod prior_artwork;
mod push;
mod windowed_decode;

use super::coverflow::CoverflowRows;
use super::snapshot::SnapshotRows;
use crate::queue::artwork::{load_artwork, ArtTarget};
use crate::queue::QueueController;

impl QueueController {
    /// Push one pulled snapshot + built coverflow onto `QueueState`. Runs the
    /// artwork-job collection synchronously, then hands off to the event loop.
    pub(in crate::queue) fn push_to_ui(&self, rows: SnapshotRows, cf: CoverflowRows, infinite: bool) {
        // Collect artwork jobs before the rows move into the closure.
        let mut art_jobs: Vec<(ArtTarget, String)> = Vec::new();
        if let Some(np) = rows.now_playing.as_ref() {
            if !np.artwork_url.is_empty() {
                art_jobs.push((ArtTarget::NowPlaying, np.artwork_url.clone()));
            }
        }
        for (idx, row) in rows.page_rows.iter().enumerate() {
            if !row.artwork_url.is_empty() {
                art_jobs.push((ArtTarget::Upcoming(idx), row.artwork_url.clone()));
            }
        }
        for (idx, row) in rows.history_rows.iter().enumerate() {
            if !row.artwork_url.is_empty() {
                art_jobs.push((ArtTarget::History(idx), row.artwork_url.clone()));
            }
        }
        // Coverflow art candidates. The COVERFLOW FAN only needs a ±4 window
        // around the current flat index (only ~9 rows can be near the visible ±3
        // fan). BUT the immersive QUEUE PANEL reuses this SAME flat model as a
        // full vertical UP-NEXT list, where EVERY visible row must show art — not
        // just the ±4 nearest (the reported "only now-playing + next 4 have art"
        // bug). So gather ALL rows with a cover here and let the event-loop
        // closure (which can read ImmersiveState) pick the range: the whole list
        // when the immersive queue panel is showing, else ±4. Either way the
        // closure decodes ONLY rows whose model cell still lacks a handle (lazy),
        // so a pure advance still decodes at most the one cover that just entered.
        const CF_WINDOW: usize = 4;
        let cf_lo = cf.index.saturating_sub(CF_WINDOW);
        let cf_hi = (cf.index + CF_WINDOW).min(cf.rows.len().saturating_sub(1));
        let mut coverflow_art_jobs: Vec<(usize, String)> = Vec::new();
        for (flat_idx, row) in cf.rows.iter().enumerate() {
            if !row.artwork_url.is_empty() {
                coverflow_art_jobs.push((flat_idx, row.artwork_url.clone()));
            }
        }

        let weak = self.weak.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            push::push_in_event_loop(&w, rows, cf, infinite, cf_lo, cf_hi, coverflow_art_jobs);
        });

        load_artwork(self.weak.clone(), art_jobs);
    }
}
