use serde::{Deserialize, Serialize};

/// One scored entity within a query bucket. `kind` is one of
/// `"artist" | "album" | "track" | "playlist"`; `id` is the entity id as a
/// string. We persist as a flat list (instead of a map keyed by a tuple)
/// because JSON object keys must be strings — a list of records round-trips
/// cleanly and is unambiguous.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ScoredEntity {
    pub(super) kind: String,
    pub(super) id: String,
    pub(super) score: i64,
}

/// One query bucket: the normalized query plus its scored entities. `order` is
/// a monotonically increasing recency stamp used for LRU eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct QueryBucket {
    pub(super) query: String,
    #[serde(default)]
    pub(super) order: u64,
    pub(super) entities: Vec<ScoredEntity>,
}

/// The full persisted document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct RankingDoc {
    #[serde(default)]
    pub(super) buckets: Vec<QueryBucket>,
}
