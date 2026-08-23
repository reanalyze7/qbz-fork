//! Pulling the core's full queue snapshot and building the now-playing +
//! filtered/paginated upcoming + history plain row data.

use qbz_models::QueueTrack;

use crate::queue::paging::paginate;
use crate::queue::row::{display_title, row_from, RowData};
use crate::queue::QueueController;

/// Everything `apply.rs` needs out of one snapshot pull, built off the UI
/// thread (all plain `Send` data).
pub(in crate::queue) struct SnapshotRows {
    pub(in crate::queue) now_playing: Option<RowData>,
    pub(in crate::queue) now_playing_favorite: bool,
    pub(in crate::queue) page_rows: Vec<RowData>,
    pub(in crate::queue) upcoming_total: usize,
    pub(in crate::queue) page: usize,
    pub(in crate::queue) page_count: usize,
    pub(in crate::queue) page_start: usize,
    pub(in crate::queue) page_end: usize,
    pub(in crate::queue) remaining: usize,
    pub(in crate::queue) history_rows: Vec<RowData>,
    pub(in crate::queue) tab: i32,
    pub(in crate::queue) stop_after_id: slint::SharedString,
    /// The unfiltered snapshot's history + upcoming, kept around for the
    /// coverflow builder (which needs the full unfiltered lists).
    pub(in crate::queue) history: Vec<QueueTrack>,
    pub(in crate::queue) upcoming: Vec<QueueTrack>,
    pub(in crate::queue) current_track: Option<QueueTrack>,
}

impl QueueController {
    /// Pull the full queue state and build the now-playing / upcoming-page /
    /// history plain row data, applying the active search filter + pagination.
    /// Also clamps + persists the paginator page back into `self.view`.
    pub(in crate::queue) async fn pull_snapshot_rows(&self) -> SnapshotRows {
        let state = self.runtime.core().get_queue_state_full().await;

        // --- NOW PLAYING --------------------------------------------------
        // Heart state comes from the shared fav_cache (disk-seeded at
        // session activation), so it is correct offline too.
        let now_playing = state.current_track.as_ref().map(|t| row_from(t, true));
        let now_playing_favorite = state
            .current_track
            .as_ref()
            .map(|t| crate::fav_cache::contains(t.id))
            .unwrap_or(false);

        // --- UP NEXT (search-filtered) -----------------------------------
        let (search, page, tab) = self
            .view
            .lock()
            .map(|v| (v.search.clone(), v.page, v.tab))
            .unwrap_or_default();
        let query = search.trim().to_lowercase();

        let filtered: Vec<&QueueTrack> = if query.is_empty() {
            state.upcoming.iter().collect()
        } else {
            state
                .upcoming
                .iter()
                .filter(|t| {
                    display_title(t).to_lowercase().contains(&query)
                        || t.artist.to_lowercase().contains(&query)
                })
                .collect()
        };

        let upcoming_total = filtered.len();
        let bounds = paginate(upcoming_total, page);
        let (page, page_count, start, end) =
            (bounds.page, bounds.page_count, bounds.start, bounds.end);
        // Persist the clamped page in case the filter shrank the list.
        if let Ok(mut view) = self.view.lock() {
            view.page = page;
        }

        let page_rows: Vec<RowData> = filtered[start..end]
            .iter()
            .map(|t| row_from(t, false))
            .collect();

        // page-start / page-end are 1-based for the human-readable counter.
        let page_start = if upcoming_total == 0 { 0 } else { start + 1 };
        let page_end = end;
        // "left" mirrors the Tauri queueRemainingTracks: tracks after the
        // current one across the whole (unfiltered) queue.
        let remaining = state
            .current_index
            .map(|idx| state.total_tracks.saturating_sub(idx + 1))
            .unwrap_or(state.total_tracks);

        // --- HISTORY ------------------------------------------------------
        let history_rows: Vec<RowData> =
            state.history.iter().map(|t| row_from(t, false)).collect();

        let stop_after_id: slint::SharedString = state
            .stop_after_track_id
            .map(|id| id.to_string())
            .unwrap_or_default()
            .into();

        SnapshotRows {
            now_playing,
            now_playing_favorite,
            page_rows,
            upcoming_total,
            page,
            page_count,
            page_start,
            page_end,
            remaining,
            history_rows,
            tab,
            stop_after_id,
            history: state.history,
            upcoming: state.upcoming,
            current_track: state.current_track,
        }
    }
}
