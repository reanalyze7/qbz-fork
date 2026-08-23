use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::schema::{QueryBucket, RankingDoc, ScoredEntity};
use super::tunables::MAX_SCORE;

/// Per-query interaction ranking. See module docs.
pub struct SearchRanking {
    /// Path to the JSON file we read/write.
    pub(super) path: PathBuf,
    /// `normalized_query -> { (kind, id) : score }`.
    pub(super) ranking: HashMap<String, HashMap<(String, String), i64>>,
    /// `normalized_query -> recency stamp` (higher = more recently touched).
    pub(super) order: HashMap<String, u64>,
    /// Monotonic counter feeding `order`.
    pub(super) tick: u64,
}

impl SearchRanking {
    /// Load the ranking from `<base_dir>/search/search_ranking.json`.
    ///
    /// A missing or corrupt file yields an empty ranking — this never panics
    /// and never returns an error. The `search/` subdir is created lazily on
    /// the first successful save, not here.
    pub fn new(base_dir: &Path) -> Self {
        let path = base_dir.join("search").join("search_ranking.json");
        let mut store = SearchRanking {
            path,
            ranking: HashMap::new(),
            order: HashMap::new(),
            tick: 0,
        };
        store.load();
        store
    }

    /// Read + parse the JSON file into memory. Any error degrades to empty.
    fn load(&mut self) {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(_) => return, // missing file == empty ranking (normal first run)
        };
        let doc: RankingDoc = match serde_json::from_str(&text) {
            Ok(d) => d,
            Err(e) => {
                log::warn!(
                    "search_ranking: corrupt JSON at {:?} ({e}); starting empty",
                    self.path
                );
                return;
            }
        };
        let mut max_order = 0u64;
        for bucket in doc.buckets {
            let mut map: HashMap<(String, String), i64> = HashMap::new();
            for ent in bucket.entities {
                let score = ent.score.clamp(0, MAX_SCORE);
                if score <= 0 {
                    continue;
                }
                map.insert((ent.kind, ent.id), score);
            }
            if map.is_empty() {
                continue;
            }
            max_order = max_order.max(bucket.order);
            self.order.insert(bucket.query.clone(), bucket.order);
            self.ranking.insert(bucket.query, map);
        }
        self.tick = max_order;
    }

    /// Serialize the current in-memory state and write it to disk. Best-effort:
    /// failures are logged and swallowed. Creates the `search/` subdir if needed.
    pub(super) fn persist(&self) {
        if let Some(parent) = self.path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::warn!(
                    "search_ranking: cannot create dir {:?} ({e}); skipping persist",
                    parent
                );
                return;
            }
        }
        let mut buckets: Vec<QueryBucket> = self
            .ranking
            .iter()
            .map(|(query, map)| {
                let mut entities: Vec<ScoredEntity> = map
                    .iter()
                    .map(|((kind, id), &score)| ScoredEntity {
                        kind: kind.clone(),
                        id: id.clone(),
                        score,
                    })
                    .collect();
                // Deterministic on-disk order: highest score first, then kind/id.
                entities.sort_by(|a, b| {
                    b.score
                        .cmp(&a.score)
                        .then_with(|| a.kind.cmp(&b.kind))
                        .then_with(|| a.id.cmp(&b.id))
                });
                QueryBucket {
                    query: query.clone(),
                    order: self.order.get(query).copied().unwrap_or(0),
                    entities,
                }
            })
            .collect();
        // Stable file output: sort buckets by query name.
        buckets.sort_by(|a, b| a.query.cmp(&b.query));

        let doc = RankingDoc { buckets };
        let json = match serde_json::to_string_pretty(&doc) {
            Ok(j) => j,
            Err(e) => {
                log::warn!("search_ranking: serialize failed ({e}); skipping persist");
                return;
            }
        };
        if let Err(e) = std::fs::write(&self.path, json) {
            log::warn!(
                "search_ranking: write to {:?} failed ({e}); state kept in memory",
                self.path
            );
        }
    }
}
