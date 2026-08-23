//! The D11.b/B8 offline visibility predicate, shared by `rebuild.rs` and
//! `folder_popup.rs`.

use super::{SidebarData, SidebarPlaylist};

/// D11.b visibility: ONLINE every playlist shows; OFFLINE the MIXED ones
/// (>= 1 local sidecar track) stay, plus — B8 — the snapshot-available ones
/// (>= 1 cached snapshot track). Everything else hides.
pub(super) fn offline_visible(data: &SidebarData, offline: bool, p: &SidebarPlaylist) -> bool {
    !offline
        || data.local_counts.get(&p.id).copied().unwrap_or(0) > 0
        || data.snapshot_available.contains(&p.id)
}
