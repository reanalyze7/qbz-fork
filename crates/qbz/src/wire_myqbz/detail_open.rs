use crate::*;

use MyQbzDetailActions as Act;

/// Open an item -> album / local-album / playlist, and open an item's
/// artist (routed by source: Qobuz numeric id vs. local name).
pub(crate) fn wire_myqbz_detail_open(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<Act>()
            .on_open_item(move |_source, item_type, source_item_id| {
                let Some(w) = weak.upgrade() else { return };
                let id = source_item_id.to_string();
                match item_type.as_str() {
                    // Album / track items both open an album view; the top-level
                    // open-album callback handles Qobuz-vs-local routing + history.
                    "album" | "track" => {
                        w.invoke_open_album(id.into());
                    }
                    "playlist" => {
                        nav::record(nav::NavEntry::Playlist(id.clone()));
                        navigate_playlist(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    other => {
                        log::warn!("[qbz-slint] myqbz_detail open-item: unknown type {other}");
                    }
                }
            });
    }

    // --- Open an item's artist (route by SOURCE) -------------------------
    {
        let weak = window.as_weak();
        window
            .global::<Act>()
            .on_open_artist(move |source, artist_name, artist_id| {
                let Some(w) = weak.upgrade() else { return };
                // The top-level open-artist callback routes a numeric id to
                // the Qobuz artist page (with nav history — the same path
                // AlbumView's artist button uses) and a name to the
                // LocalLibrary Artists tab. Stored items only carry the
                // artist NAME, so Qobuz rows route by the numeric artist id
                // the resolveItems pass derived from their first track.
                if source == "qobuz" {
                    if !artist_id.trim().is_empty() {
                        w.invoke_open_artist(artist_id);
                    } else {
                        // Resolve still pending (or failed) — do NOT fall
                        // back to the name: that opens the WRONG page (the
                        // LocalLibrary artist) for a Qobuz item.
                        log::warn!(
                            "[qbz-slint] myqbz_detail open-artist: qobuz item '{artist_name}' \
                             has no resolved artist id yet — ignoring click"
                        );
                    }
                } else if !artist_name.trim().is_empty() {
                    // local -> the LocalLibrary Artists tab by NAME.
                    w.invoke_open_artist(artist_name);
                }
            });
    }
}
