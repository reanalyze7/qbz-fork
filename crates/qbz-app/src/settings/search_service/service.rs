use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::super::search_cache::SearchCache;
use super::super::search_ranking::SearchRanking;
use super::InteractionAction;

/// The composed Intelligent Search service: cache (Capa A) + ranking (Capa B),
/// gated by an `enabled` kill switch. Frontend-agnostic, headless, plain (no
/// interior `Mutex` around the stores — the caller locks).
pub struct SearchService {
    /// Capa A — result cache (SWR).
    cache: SearchCache,
    /// Capa B — per-query interaction ranking.
    ranking: SearchRanking,
    /// Master on/off. Default `true`. When `false`, every method is an inert
    /// no-op (`cached`/`top_for_query` return `None`; `store`/`record_interaction`/
    /// `rank_within` do nothing).
    enabled: AtomicBool,
}

impl SearchService {
    /// Construct both stores rooted at `base_dir` (typically the per-user data
    /// dir). Each store owns its own sub-file / sub-dir under that base. Never
    /// fails: missing or corrupt persisted state degrades to empty.
    pub fn new(base_dir: &Path) -> Self {
        Self {
            cache: SearchCache::new(base_dir),
            ranking: SearchRanking::new(base_dir),
            enabled: AtomicBool::new(true),
        }
    }

    /// Whether Intelligent Search is currently enabled.
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Flip the master kill switch. Works through a shared `&self` so the toggle
    /// can be applied without taking the store lock for mutation.
    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }

    /// Cached merged result for `query`, or `None` when disabled or uncached.
    pub fn cached(&self, query: &str) -> Option<qbz_models::SearchAllResults> {
        if !self.enabled() {
            return None;
        }
        self.cache.get(query)
    }

    /// Store a live `results` page for `query` in the cache. No-op when disabled.
    pub fn store(&mut self, query: &str, results: &qbz_models::SearchAllResults) {
        if !self.enabled() {
            return;
        }
        self.cache.put(query, results);
    }

    /// Record a user interaction with a search-surfaced entity. No-op when
    /// disabled. `kind` is one of `"artist" | "album" | "track" | "playlist"`.
    pub fn record_interaction(
        &mut self,
        query: &str,
        kind: &str,
        id: &str,
        action: InteractionAction,
    ) {
        if !self.enabled() {
            return;
        }
        self.ranking.record(query, kind, id, action);
    }

    /// The single highest-scored `(kind, id)` learned for `query`, or `None`
    /// when disabled / nothing learned.
    pub fn top_for_query(&self, query: &str) -> Option<(String, String)> {
        if !self.enabled() {
            return None;
        }
        self.ranking.top_for_query(query)
    }

    /// Stable-sort `items` in place by their learned score for `query` (the
    /// cortinilla reorder). No-op when disabled.
    pub fn rank_within<T>(
        &self,
        query: &str,
        kind: &str,
        items: &mut Vec<T>,
        id_of: impl Fn(&T) -> String,
    ) {
        if !self.enabled() {
            return;
        }
        self.ranking.rank_within(query, kind, items, id_of);
    }
}
