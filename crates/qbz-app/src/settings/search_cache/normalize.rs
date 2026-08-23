/// Normalize a raw query into the canonical cache key: lowercase, trimmed,
/// internal whitespace runs collapsed to single spaces.
///
/// This is the ONLY definition of the cache key; `search_service.rs` and
/// `search_ranking.rs` import it from here.
pub fn normalize_query(q: &str) -> String {
    q.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
