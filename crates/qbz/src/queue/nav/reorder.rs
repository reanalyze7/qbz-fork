//! Drag-reorder + the page-local <-> queue-wide upcoming index resolution
//! it (and the play/remove callbacks in `mod.rs`) shares.

use crate::queue::paging::paginate;
use crate::queue::row::display_title;
use crate::queue::QueueController;

impl QueueController {
    /// Reorder the upcoming list: move the page-local row `from_page` to the
    /// insertion slot `to_slot` (0..=page_len). Resolves BOTH to queue-wide
    /// upcoming indices (honoring the search filter), then routes: connected ->
    /// the QConnect cloud (WS-authoritative, the cloud owns queue order); offline
    /// -> core `move_track` (already shuffle-aware) + refresh.
    pub fn reorder(&self, from_page: usize, to_slot: usize) {
        let this = self.clone();
        self.handle.spawn(async move {
            let Some(from_q) = this.resolve_upcoming_index(from_page).await else {
                log::warn!("[qbz-slint] queue: reorder from {from_page} out of range");
                return;
            };
            // `to_slot` is an insertion slot in [0, page_len]. Slot k<page_len sits
            // before page-local row k; slot==page_len appends after the last
            // visible row (one past it in queue-wide upcoming space).
            let page_len = this.current_page_len().await;
            let to_q = if to_slot >= page_len {
                match this.resolve_upcoming_index(page_len.saturating_sub(1)).await {
                    Some(last) => last + 1,
                    None => return,
                }
            } else {
                match this.resolve_upcoming_index(to_slot).await {
                    Some(idx) => idx,
                    None => return,
                }
            };
            if from_q == to_q {
                return;
            }

            // Connected -> the cloud reorders and echoes a QueueUpdated that
            // materialize applies; do NOT also reorder locally (would diverge).

            this.runtime.core().move_track(from_q, to_q).await;
            this.refresh_async().await;
        });
    }

    /// Number of upcoming rows on the current (filtered) page — used to detect
    /// the "append after last" insertion slot in `reorder`.
    pub(in crate::queue) async fn current_page_len(&self) -> usize {
        let (search, page) = self
            .view
            .lock()
            .map(|v| (v.search.clone(), v.page))
            .unwrap_or_default();
        let query = search.trim().to_lowercase();
        let state = self.runtime.core().get_queue_state_full().await;
        let total = if query.is_empty() {
            state.upcoming.len()
        } else {
            state
                .upcoming
                .iter()
                .filter(|t| {
                    display_title(t).to_lowercase().contains(&query)
                        || t.artist.to_lowercase().contains(&query)
                })
                .count()
        };
        let bounds = paginate(total, page);
        bounds.end - bounds.start
    }

    /// Resolve a page-local upcoming index to a queue-wide upcoming index.
    /// When a search is active the queue-wide index is found by matching
    /// the filtered row back against the unfiltered upcoming list.
    pub(in crate::queue) async fn resolve_upcoming_index(&self, page_index: usize) -> Option<usize> {
        let (search, page) = self.view.lock().map(|v| (v.search.clone(), v.page)).ok()?;
        let query = search.trim().to_lowercase();
        let absolute_in_filtered = page * crate::queue::PAGE_SIZE + page_index;

        if query.is_empty() {
            return Some(absolute_in_filtered);
        }

        // Walk the unfiltered upcoming list, counting matches until the
        // requested filtered position is reached.
        let state = self.runtime.core().get_queue_state_full().await;
        let mut matched = 0usize;
        for (idx, track) in state.upcoming.iter().enumerate() {
            let hit = display_title(track).to_lowercase().contains(&query)
                || track.artist.to_lowercase().contains(&query);
            if hit {
                if matched == absolute_in_filtered {
                    return Some(idx);
                }
                matched += 1;
            }
        }
        None
    }
}
