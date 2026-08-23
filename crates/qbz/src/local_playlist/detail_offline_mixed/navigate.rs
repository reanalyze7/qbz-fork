use slint::ComponentHandle;

use super::apply::apply_qobuz_offline;
use super::gather::{gather_sidecar, resolve_playable};
use crate::artwork::{self, ImageCache};
use crate::local_playlist::detail_local::artwork_jobs;
use crate::{AppWindow, ContentView, NavState, PlaylistState};

/// Open the OFFLINE rendering of a MIXED (Qobuz-id) playlist: the playlist's
/// SNAPSHOT membership rows that are playable offline (B8: snapshot ∩
/// cached, grace-gated, resolved from the offline-cache index like the
/// LOCAL detail's Cached rows), then its local sidecar rows
/// (`playlist_local_tracks`).
///
/// MERGE RULE: the Qobuz block renders FIRST in snapshot position order,
/// then the sidecar block in sidecar position order — the sidecar positions
/// are absolute slots assigned AFTER the Qobuz block by the online append
/// convention, so block-then-block keeps each source's own order without
/// trusting cross-source position arithmetic against a cached-only Qobuz
/// subset. A track present both in the snapshot and as a sidecar local row
/// renders twice, exactly like the online detail does.
///
/// The name/description come from the sidebar's last-loaded session cache,
/// else the persisted snapshot name (B7 — survives a cold offline start).
pub fn navigate_qobuz_offline(
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
    playlist_id: u64,
) {
    handle.spawn(async move {
        {
            let weak = weak.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                crate::playlist::reset(&w);
                // Set BEFORE the view switch so the AppShell mounts the
                // detail (not the OfflinePlaceholder) with no flash.
                w.global::<PlaylistState>().set_offline_subset(true);
                crate::sidebar::set_active(&w, &playlist_id.to_string());
                w.global::<NavState>().set_view(ContentView::Playlist);
            });
        }

        let (sidecar_rows, custom_artwork_path, playable_ids, snapshot_name) =
            tokio::task::spawn_blocking(move || gather_sidecar(playlist_id))
                .await
                .unwrap_or_default();

        let mut rows = resolve_playable(&playable_ids).await;
        // Merge rule (see the doc comment): Qobuz snapshot block first,
        // sidecar block after, each in its own position order.
        rows.extend(sidecar_rows);

        let (name, description) = crate::sidebar::playlist_name_desc(playlist_id)
            .or_else(|| snapshot_name.map(|n| (n, String::new())))
            .unwrap_or_else(|| ("Playlist".to_string(), String::new()));
        let (http_jobs, local_jobs) = artwork_jobs(&rows);
        let _ = weak.upgrade_in_event_loop(move |w| {
            apply_qobuz_offline(&w, playlist_id, name, description, custom_artwork_path, rows);
        });
        // Sidecar rows carry file paths — the http set stays empty, kept
        // for symmetry with the local detail.
        if !http_jobs.is_empty() {
            artwork::spawn_loads(http_jobs, weak.clone(), image_cache.clone());
        }
        if !local_jobs.is_empty() {
            artwork::spawn_local_loads(local_jobs, weak.clone(), image_cache.clone());
        }
    });
}
