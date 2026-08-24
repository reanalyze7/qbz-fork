//! The `ContentView::Playlist` branch of `play_track_in_context`.
use slint::ComponentHandle;

use crate::playback::queue_build::from_model::order_by_visible;
use crate::playback::queue_build::play_queue::play_tracks_ctx;
use crate::playback::Runtime;
use crate::{AppWindow, PlaylistState};

/// LOCAL playlist detail (id "local:<uuid>") — queue from its
/// own resolved snapshot + the D8 offline-only stamp. The
/// offline sidecar rendering of a MIXED playlist (D11.a) plays
/// from the same snapshot (its rows resolve locally), and so
/// does the ONLINE mixed detail (Seam B: source-aware
/// QueueTracks; QConnect admission rejects the non-Qobuz rows
/// per-track at push time). The now-playing context stays
/// ("playlist", id) — anything Qobuz-bound that reads it
/// re-resolves Qobuz membership, so sidecar rows are excluded
/// from the context by construction (Tauri :1825 parity).
pub(super) fn handle(
    window: &AppWindow,
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    if window.global::<PlaylistState>().get_is_local()
        || window.global::<PlaylistState>().get_offline_subset()
        || crate::playlist::is_mixed()
    {
        if crate::local_playlist::play_from_visible(
            window,
            runtime.clone(),
            weak.clone(),
            handle.clone(),
            clicked_id,
        ) {
            return true;
        }
    } else if let Some((tracks, idx)) = order_by_visible(
        &window.global::<PlaylistState>().get_tracks(),
        crate::playlist::current_tracks(),
        clicked_id,
    ) {
        let ctx_id = window.global::<PlaylistState>().get_id().to_string();
        play_tracks_ctx(
            runtime.clone(),
            weak.clone(),
            handle.clone(),
            tracks,
            idx,
            Some(("playlist".to_string(), ctx_id)),
        );
        return true;
    }
    false
}
