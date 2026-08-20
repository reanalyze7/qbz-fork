//! Async playlist-list fetch for the picker (see `playlist_picker.rs` for
//! the module split rationale). Split out purely to keep `playlist_picker.rs`
//! under the 130-line budget.

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::playlist_picker::{pending_snapshot, PickPlaylist};

/// Fetch the user's playlists (worker thread) plus, per row, whether the
/// pending ids/refs are already all present (checkbox state). LOCAL
/// playlists (library.db — always available) come first, then the Qobuz set
/// (skipped entirely while OFFLINE — D3/D11, Qobuz playlists can't be
/// written to offline).
pub async fn load<A>(runtime: &AppRuntime<A>) -> Vec<PickPlaylist>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let (pending, local_mode) = pending_snapshot();
    let mut out: Vec<PickPlaylist> = tokio::task::spawn_blocking({
        let pending = pending.clone();
        move || {
            crate::local_playlist::list_blocking()
                .into_iter()
                .map(|p| {
                    let already_has =
                        crate::playlist_membership::already_has_blocking(&p.id, &pending, local_mode);
                    PickPlaylist { id: p.id, name: p.name, tracks: p.track_count, is_local: true, already_has }
                })
                .collect::<Vec<_>>()
        }
    })
    .await
    .unwrap_or_default();

    if crate::offline_mode::engine().is_offline() {
        return out;
    }
    match runtime.core().get_user_playlists().await {
        Ok(playlists) => {
            for p in playlists {
                let pid = p.id;
                let already_has = if local_mode {
                    let refs = pending.clone();
                    tokio::task::spawn_blocking(move || {
                        crate::playlist_membership_qobuz::already_has_refs_blocking(pid, &refs)
                    })
                    .await
                    .unwrap_or(false)
                } else {
                    let ids: Vec<u64> = pending.iter().filter_map(|s| s.parse::<u64>().ok()).collect();
                    if ids.is_empty() {
                        false
                    } else {
                        match runtime.core().check_playlist_duplicates(pid, &ids).await {
                            Ok(dup) => dup.duplicate_count as usize == ids.len(),
                            Err(_) => false,
                        }
                    }
                };
                out.push(PickPlaylist {
                    id: pid.to_string(),
                    name: p.name,
                    tracks: p.tracks_count,
                    is_local: false,
                    already_has,
                });
            }
        }
        Err(e) => {
            log::warn!("[qbz-slint] playlist picker load failed: {e}");
        }
    }
    out
}
