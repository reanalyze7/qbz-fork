//! Async fetch + merge: `QbzCore::get_playlist` -> `PlaylistData`.

mod data;

pub use data::PlaylistData;
pub(crate) use data::interleave_rows;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use data::truncate_words;

pub async fn load<A>(runtime: &AppRuntime<A>, playlist_id: u64) -> Option<PlaylistData>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let pl = match runtime.core().get_playlist(playlist_id).await {
        Ok(pl) => pl,
        Err(e) => {
            log::error!("[qbz-slint] load playlist {playlist_id} failed: {e}");
            return None;
        }
    };
    let tracks = pl.tracks.map(|c| c.items).unwrap_or_default();
    // Header cover: the server-composed playlist image, else the first
    // track's album cover.
    let cover_url = pl
        .images
        .as_ref()
        .and_then(|imgs| imgs.first().cloned())
        .or_else(|| {
            tracks
                .first()
                .and_then(|t| t.album.as_ref())
                .and_then(|a| a.image.best().cloned())
        })
        .unwrap_or_default();
    // Local custom artwork (shared with the Tauri app via library.db).
    let custom_artwork_path = tokio::task::spawn_blocking(move || {
        crate::library_db::with_db(|db| db.get_playlist_settings(playlist_id))
            .flatten()
            .and_then(|s| s.custom_artwork_path)
            .filter(|p| !p.is_empty())
    })
    .await
    .ok()
    .flatten();
    let description = pl
        .description
        .map(|d| crate::strip_html::strip_html(&d))
        .unwrap_or_default();
    // B7 producer (membership): this fetch already returned the FULL track
    // list — full-replace the playlist's snapshot membership, detached (the
    // render never waits). No-ops for playlists outside the user's listed
    // set (no snapshot header), so merely-viewed public playlists stay out.
    // Qobuz membership only — sidecar rows never enter the snapshot (E10).
    crate::playlist_snapshot::record_detail_detached(
        playlist_id,
        pl.name.clone(),
        pl.owner.name.clone(),
        tracks.iter().map(|t| t.id).collect(),
    );
    // Seam A (merge-on-load): read the sidecar rows (healing inside the
    // shared reader) and interleave them with the Qobuz tracks at their
    // absolute slots.
    let qobuz_count = tracks.len() as u32;
    let sidecar = tokio::task::spawn_blocking(move || {
        crate::local_playlist::read_sidecar_rows_blocking(playlist_id, qobuz_count)
    })
    .await
    .unwrap_or_default();
    let rows = interleave_rows(tracks, sidecar);
    Some(PlaylistData {
        id: pl.id.to_string(),
        name: pl.name,
        owner_id: pl.owner.id,
        owner: pl.owner.name,
        description: description.clone(),
        description_short: truncate_words(&description, 160),
        cover_url,
        custom_artwork_path,
        rows,
    })
}
