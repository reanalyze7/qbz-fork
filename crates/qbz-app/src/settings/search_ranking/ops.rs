use super::action::InteractionAction;
use super::store::SearchRanking;
use super::tunables::{MAX_QUERIES, MAX_SCORE};
use crate::settings::search_cache::normalize_query;

impl SearchRanking {
    /// Touch a query's recency stamp (call when a query bucket is created or
    /// mutated). Returns the new stamp.
    fn touch(&mut self, query: &str) -> u64 {
        self.tick += 1;
        self.order.insert(query.to_string(), self.tick);
        self.tick
    }

    /// Evict the least-recently-touched query when over the LRU cap.
    fn enforce_query_cap(&mut self) {
        while self.ranking.len() > MAX_QUERIES {
            // Find the query with the smallest recency stamp.
            let victim = self
                .order
                .iter()
                .filter(|(q, _)| self.ranking.contains_key(*q))
                .min_by_key(|(_, &stamp)| stamp)
                .map(|(q, _)| q.clone());
            match victim {
                Some(q) => {
                    self.ranking.remove(&q);
                    self.order.remove(&q);
                }
                None => break, // defensive: nothing to evict
            }
        }
    }

    /// Record an interaction: bump `(kind, id)`'s score for `query` by the
    /// action weight, cap it at `MAX_SCORE`, enforce the LRU query cap, then
    /// persist (best-effort). `kind` should be one of
    /// `"artist" | "album" | "track" | "playlist"`.
    pub fn record(&mut self, query: &str, kind: &str, id: &str, action: InteractionAction) {
        let key = normalize_query(query);
        if key.is_empty() {
            return;
        }
        self.touch(&key);
        let bucket = self.ranking.entry(key.clone()).or_default();
        let slot = bucket.entry((kind.to_string(), id.to_string())).or_insert(0);
        *slot = (*slot + action.weight()).min(MAX_SCORE);
        self.enforce_query_cap();
        self.persist();
    }

    /// The single highest-scored entity for `query`, if any. Ties break
    /// deterministically: higher score, then kind ascending, then id ascending.
    pub fn top_for_query(&self, query: &str) -> Option<(String, String)> {
        let key = normalize_query(query);
        let bucket = self.ranking.get(&key)?;
        bucket
            .iter()
            .max_by(|(ak, &asc), (bk, &bsc)| {
                // We want the *max* element; for ties we prefer the lexically
                // smaller (kind, id), so invert those comparisons.
                asc.cmp(&bsc)
                    .then_with(|| bk.0.cmp(&ak.0))
                    .then_with(|| bk.1.cmp(&ak.1))
            })
            .map(|((kind, id), _)| (kind.clone(), id.clone()))
    }

    /// The learned score for a specific `(kind, id)` under `query`, or 0.
    pub fn score_for(&self, query: &str, kind: &str, id: &str) -> i64 {
        let key = normalize_query(query);
        self.ranking
            .get(&key)
            .and_then(|b| b.get(&(kind.to_string(), id.to_string())))
            .copied()
            .unwrap_or(0)
    }

    /// Stable-sort `items` in place, descending by their learned score for
    /// `(kind, id_of(item))` under `query`. Items with no learned score keep
    /// their original relative order and sit behind all scored items.
    ///
    /// This is for the **cortinilla only** — never call it to reorder the
    /// results page.
    pub fn rank_within<T>(
        &self,
        query: &str,
        kind: &str,
        items: &mut Vec<T>,
        id_of: impl Fn(&T) -> String,
    ) {
        let key = normalize_query(query);
        let bucket = match self.ranking.get(&key) {
            Some(b) if !b.is_empty() => b,
            _ => return, // nothing learned for this query: leave API order intact
        };
        // `sort_by` is stable, so equal-score items (including all unscored,
        // which share score 0) retain their original relative order. Descending
        // by score puts scored items ahead of unscored ones.
        items.sort_by(|a, b| {
            let sa = bucket
                .get(&(kind.to_string(), id_of(a)))
                .copied()
                .unwrap_or(0);
            let sb = bucket
                .get(&(kind.to_string(), id_of(b)))
                .copied()
                .unwrap_or(0);
            sb.cmp(&sa)
        });
    }
}
