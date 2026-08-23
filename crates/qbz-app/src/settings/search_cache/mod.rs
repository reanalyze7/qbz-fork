//! CAPA A — stale-while-revalidate result cache for Intelligent Search.
//!
//! Frontend-agnostic (ADR-006) cache layer for combined search results.
//! It owns ONLY the caching of `SearchAllResults` and exposes a tiny,
//! synchronous `get`/`put` surface. It does NOT hold a `QbzCore`, does NOT
//! call `core.search_all`, and knows nothing about the live network fetch.
//! The SWR orchestration (render cached → fire live → replace, guarded by a
//! version counter) lives in the qbz-slint controller, which already calls
//! `core.search_all()` itself.
//!
//! ## Two tiers, by volatility
//!
//! - **Volatile (albums / tracks / playlists):** an in-memory, insertion-order
//!   LRU bounded to [`VOLATILE_CACHE_CAPACITY`] queries. These categories are
//!   new-release-sensitive, so they are intentionally NOT persisted — a fresh
//!   app launch starts them empty and the first live fetch repopulates them.
//! - **Persisted (artists):** a small JSON store (`<base>/search_artist_cache.json`)
//!   mapping a normalized query → its cached `Vec<Artist>`. Artists change far
//!   less often than album/track/playlist catalogs, so persisting them lets a
//!   repeated query return its artist slice instantly across restarts. The store
//!   degrades gracefully: a missing or corrupt file simply starts empty and is
//!   overwritten on the next `put`, never a panic.
//!
//! ## The cache key
//!
//! [`normalize_query`] is THE single source of truth for the key: lowercased,
//! trimmed, with internal runs of whitespace collapsed to single spaces. Both
//! the volatile LRU and the persisted artist store key on the same normalized
//! string, and `search_service.rs` / `search_ranking.rs` re-use this function
//! (there is exactly ONE definition, here).

mod artist_store;
mod cache;
mod normalize;
#[cfg(test)]
mod tests;

/// Max distinct queries held in the volatile (album/track/playlist) LRU.
/// The spec calls for "~40 queries"; the oldest is evicted past this bound.
pub const VOLATILE_CACHE_CAPACITY: usize = 40;

/// On-disk filename for the persisted artist slice (under the per-user base dir).
pub const ARTIST_CACHE_FILE: &str = "search_artist_cache.json";

pub use cache::SearchCache;
pub use normalize::normalize_query;
