/// Weight added for an "open" / navigate interaction.
pub const WEIGHT_OPEN: i64 = 1;
/// Weight added for a "play" interaction.
pub const WEIGHT_PLAY: i64 = 2;
/// Weight added for a "favorite" interaction.
pub const WEIGHT_FAVORITE: i64 = 3;

/// Maximum accumulated score for any single `(kind, id)` pair. Prevents a
/// single hammered entity from dominating and bounds the on-disk integer size.
pub const MAX_SCORE: i64 = 1000;

/// Maximum number of distinct normalized queries retained. LRU-evicted.
pub const MAX_QUERIES: usize = 200;
