//! Fetch playlists (Qobuz) + folders + folder membership (local). The
//! Qobuz-fetch half lives in `load_playlists.rs`, the blocking-DB half in
//! `load_meta.rs`, and the offline synthesis tail in `load_offline.rs`.

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::load_meta::load_folders_and_locals;
use super::load_offline::synthesize_offline_playlists;
use super::load_playlists::fetch_playlists;
use super::{SidebarData, NAME_DESC};

/// Fetch playlists (Qobuz) + folders + folder membership (local).
pub async fn load<A>(runtime: &AppRuntime<A>) -> SidebarData
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let playlists = fetch_playlists(runtime).await;

    // Folders (hidden folders excluded) + folder membership +
    // per-playlist custom-sort positions + hidden-playlist set + the
    // first-class LOCAL playlists + the per-playlist local sidecar counts
    // (all local, library.db) + OFFLINE only: the playlist-snapshot names
    // and the snapshot-available set (B7/B8).
    let (
        folders,
        folder_map,
        positions,
        hidden_playlists,
        local_playlists,
        local_counts,
        snapshot_names,
        snapshot_available,
    ) = tokio::task::spawn_blocking(load_folders_and_locals)
        .await
        .unwrap_or_default();

    // Resolve up to 4 cover refs per LOCAL playlist for the sidebar micro-collage
    // (no network — local/cached-Qobuz covers from the playlist's tracks).
    // Done here in the async load (off the blocking DB closure above) so each
    // resolved set is cached in SidebarData; rebuild() reuses it without
    // re-resolving. Empty result = the row keeps its hard-drive glyph.
    let mut local_playlists = local_playlists;
    for lp in local_playlists.iter_mut() {
        lp.cover_urls = crate::local_playlist::resolve_cover_urls(&lp.id, 4).await;
    }

    let mut playlists = playlists;
    // D11.b — OFFLINE: the Qobuz fetch above is gate-refused (empty), so the
    // reachable playlists are synthesized locally (see `load_offline.rs`).
    if crate::offline_mode::engine().is_offline() {
        synthesize_offline_playlists(&mut playlists, &local_counts, &snapshot_available, &snapshot_names);
    }
    for p in &mut playlists {
        if let Some(pos) = positions.get(&p.id) {
            p.position = *pos;
        }
    }
    // Cache the loaded playlists' name+description for the sidebar
    // context-menu edit modal (no extra fetch on right-click).
    if let Ok(mut nd) = NAME_DESC.lock() {
        nd.clear();
        for p in &playlists {
            nd.insert(p.id, (p.name.clone(), p.description.clone()));
        }
    }
    SidebarData {
        playlists,
        folders,
        folder_map,
        hidden_playlists,
        local_playlists,
        local_counts,
        snapshot_available,
    }
}
