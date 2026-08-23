//! Custom (user-supplied) playlist artwork: copy into the artwork cache and
//! store/clear the path in `playlist_settings`. Blocking — run on a worker
//! thread.

/// Copy `src` into the artwork cache and store it as this playlist's
/// custom artwork (shared with Tauri via library.db). Returns the
/// stored path. Blocking — run on a worker thread.
pub fn set_custom_artwork(playlist_id: u64, src: &str) -> Option<String> {
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
    let dest = cache.join(format!("playlist_{playlist_id}_{ts}.{ext}"));
    if let Err(e) = std::fs::copy(src, &dest) {
        log::error!("[qbz-slint] copy custom artwork failed: {e}");
        return None;
    }
    let dest_str = dest.to_string_lossy().to_string();
    crate::library_db::with_db(|db| db.update_playlist_artwork(playlist_id, Some(&dest_str)))?;
    Some(dest_str)
}

/// Clear this playlist's custom artwork. Blocking.
pub fn clear_custom_artwork(playlist_id: u64) {
    crate::library_db::with_db(|db| db.update_playlist_artwork(playlist_id, None));
}
