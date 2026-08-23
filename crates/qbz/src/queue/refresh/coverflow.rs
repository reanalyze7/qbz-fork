//! Building the flat coverflow model + the sequence-hash gate that decides
//! whether the model needs rebuilding or just the index needs to move.
//!
//! KEY INVARIANT (queue's single most load-bearing one): on a pure
//! advance/jump the id-sequence is unchanged, so the flat model must NOT be
//! replaced — only `coverflow_index` moves. This is what avoids Repeater
//! rebuilds / re-decodes. `refresh_async`'s `apply.rs` step depends on
//! `seq_changed` being computed here, BEFORE the event-loop closure.

use qbz_models::QueueTrack;

use crate::queue::row::row_from;
use crate::queue::QueueController;
use crate::queue::row::RowData;

/// The flat coverflow row list + whether the sequence changed since the
/// last push (gates `set_coverflow_tracks` in `apply.rs`).
pub(in crate::queue) struct CoverflowRows {
    pub(in crate::queue) rows: Vec<RowData>,
    pub(in crate::queue) index: usize,
    pub(in crate::queue) seq_hash: u64,
    pub(in crate::queue) seq_changed: bool,
}

impl QueueController {
    /// Build the ONE stable flat coverflow model from the UNFILTERED queue,
    /// oldest-first: `[history.reversed (oldest..newest), NOW-PLAYING,
    /// upcoming...]`. `QueueStateFull.history` is most-recent-first, so it's
    /// reversed for the oldest-first flat order. The flat index of NOW is the
    /// number of history entries (it sits right after the reversed history).
    pub(in crate::queue) fn build_coverflow(
        &self,
        history: &[QueueTrack],
        current_track: Option<&QueueTrack>,
        upcoming: &[QueueTrack],
    ) -> CoverflowRows {
        let mut rows: Vec<RowData> = Vec::with_capacity(history.len() + 1 + upcoming.len());
        for t in history.iter().rev() {
            rows.push(row_from(t, false));
        }
        let index: usize = if let Some(t) = current_track {
            let idx = rows.len();
            rows.push(row_from(t, true));
            idx
        } else {
            // No current track: index points at the first upcoming (or 0 when
            // the whole queue is empty). The flat list is history ++ upcoming.
            rows.len()
        };
        for t in upcoming.iter() {
            rows.push(row_from(t, false));
        }

        // Order-sensitive rolling fingerprint over the flat id-sequence. Used to
        // gate the model rebuild: equal hash => same membership+order => pure
        // advance/jump => skip set_coverflow_tracks. Hashing the ordered ids
        // (not a set) catches shuffle/reorder; folding the index-free id list
        // keeps a same-sequence-different-pointer advance hashing identical.
        let seq_hash: u64 = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            rows.len().hash(&mut h);
            for r in &rows {
                r.id.hash(&mut h);
            }
            h.finish()
        };
        // Decide rebuild-vs-index-only BEFORE the event loop so the art-job set
        // can be narrowed to the ±4 window. `seq_changed` true => rebuild path.
        let seq_changed = {
            let mut last = self.last_coverflow_seq.lock().ok();
            let changed = match last.as_deref() {
                Some(Some(prev)) => *prev != seq_hash,
                _ => true, // None lock or first push -> rebuild
            };
            if let Some(slot) = last.as_deref_mut() {
                *slot = Some(seq_hash);
            }
            changed
        };

        CoverflowRows {
            rows,
            index,
            seq_hash,
            seq_changed,
        }
    }
}
