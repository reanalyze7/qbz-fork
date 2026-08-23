use qbz_library::local_playlists as repo;

use super::crud::get_tracks_blocking;

/// Resolve up to `limit` cover refs for a local playlist's tracks, in track
/// order, WITHOUT any network — for the sidebar micro-collage. Sources, all
/// local: a Local track's `local_tracks.artwork_path`, and a Qobuz track's
/// offline-cache `cover.jpg` when it is downloaded. Returns file paths (the
/// sidebar art loader routes by shape). A purely-online (uncached) Qobuz
/// playlist resolves nothing here (no network in the sidebar) and falls back
/// to the glyph. The library.db lookups run on a blocking thread; the
/// cached-Qobuz cover lives behind the offline cache's async lock.
pub async fn resolve_cover_urls(id: &str, limit: usize) -> Vec<String> {
    let pid = id.to_string();
    let (mut covers, qobuz_ids): (Vec<String>, Vec<u64>) =
        tokio::task::spawn_blocking(move || {
            let mut covers: Vec<String> = Vec::new();
            let mut qobuz_ids: Vec<u64> = Vec::new();
            for t in get_tracks_blocking(&pid) {
                if covers.len() >= limit {
                    break;
                }
                match t.source {
                    repo::LocalPlaylistTrackSource::Local => {
                        if let Some(path) = t.local_path {
                            if let Some(Some(track)) =
                                crate::library_db::with_db(|db| db.get_track_by_path(&path))
                            {
                                if let Some(art) = track.artwork_path {
                                    if !covers.contains(&art) {
                                        covers.push(art);
                                    }
                                }
                            }
                        }
                    }
                    repo::LocalPlaylistTrackSource::Qobuz => {
                        if let Some(tid) = t.qobuz_track_id {
                            qobuz_ids.push(tid);
                        }
                    }
                }
            }
            covers.truncate(limit);
            (covers, qobuz_ids)
        })
        .await
        .unwrap_or_default();

    // Fill any remaining slots from offline-cached Qobuz covers (async lock).
    if covers.len() < limit && !qobuz_ids.is_empty() {
        if let Some(off) = crate::offline::get().await {
            let cache_path = off.get_cache_path();
            let guard = off.db.lock().await;
            if let Some(db) = guard.as_ref() {
                for tid in qobuz_ids {
                    if covers.len() >= limit {
                        break;
                    }
                    if let Ok(Some(info)) = db.get_track(tid) {
                        if let Some(art) = info.resolve_cover_path(&cache_path) {
                            if !covers.contains(&art) {
                                covers.push(art);
                            }
                        }
                    }
                }
            }
        }
    }
    covers.truncate(limit);
    covers
}

/// Copy `src` into the artwork cache and store it as this local playlist's
/// custom artwork (mirrors `playlist::set_custom_artwork` for Qobuz ones).
/// Returns the stored path. Blocking.
pub fn set_custom_artwork_blocking(id: &str, src: &str) -> Option<String> {
    let cache = crate::library_db::artwork_cache_dir()?;
    std::fs::create_dir_all(&cache).ok()?;
    let ext = std::path::Path::new(src)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let suffix = id.trim_start_matches(repo::LOCAL_PLAYLIST_PREFIX);
    let dest = cache.join(format!("local_playlist_{suffix}_{ts}.{ext}"));
    if let Err(e) = std::fs::copy(src, &dest) {
        log::error!("[qbz-slint] copy local playlist artwork failed: {e}");
        return None;
    }
    let dest_str = dest.to_string_lossy().to_string();
    match crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::set_custom_artwork(conn, id, Some(&dest_str))))
    }) {
        Some(Ok(())) => Some(dest_str),
        Some(Err(e)) => {
            log::error!("[qbz-slint] store local playlist artwork failed: {e}");
            None
        }
        None => None,
    }
}

/// Clear this local playlist's custom artwork. Blocking.
pub fn clear_custom_artwork_blocking(id: &str) {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::set_custom_artwork(conn, id, None)))
    });
}
