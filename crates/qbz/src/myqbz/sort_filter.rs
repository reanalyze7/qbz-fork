//! Pure sort / filter helpers — no Slint dependency.

use qbz_models::mixtape::MixtapeCollection;

/// Sort a collection list by the active toolbar sort (mirrors `visibleX`):
/// name (locale-ish), items (count), updated (updated_at), position (default).
/// `dir` = "asc"/"desc".
pub(super) fn sort_collections(list: &mut [MixtapeCollection], sort: &str, dir: &str) {
    let desc = dir == "desc";
    match sort {
        "name" => list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        "items" => list.sort_by(|a, b| a.items.len().cmp(&b.items.len())),
        "updated" => list.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
        // default "position"
        _ => list.sort_by(|a, b| a.position.cmp(&b.position)),
    }
    if desc {
        list.reverse();
    }
}

/// Whether a collection passes the search filter (name OR description,
/// case-insensitive substring). Empty query = pass.
pub(super) fn passes_search(c: &MixtapeCollection, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    if c.name.to_lowercase().contains(query) {
        return true;
    }
    c.description
        .as_deref()
        .map(|d| d.to_lowercase().contains(query))
        .unwrap_or(false)
}
