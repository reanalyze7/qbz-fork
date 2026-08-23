//! Pure level/search predicate used by both `refresh.rs` fns.

/// Maximum rows pushed to the viewer after filtering (the ring holds up to
/// `qbz_log::ring::RING_CAP`; the view shows the most recent slice).
pub(super) const MAX_VIEW_ROWS: usize = 1000;

/// Whether `line` passes the level + search filters currently set on the global.
/// `level` is the lowercased `filter-level` ("all" = no level filter); `search`
/// is the lowercased query (empty = no search filter), matched over target +
/// message.
pub(super) fn line_matches(line: &qbz_log::LogLine, level: &str, search: &str) -> bool {
    let level_ok = level == "all" || line.level_str().eq_ignore_ascii_case(level);
    let search_ok = search.is_empty()
        || line.target.to_lowercase().contains(search)
        || line.message.to_lowercase().contains(search);
    level_ok && search_ok
}
