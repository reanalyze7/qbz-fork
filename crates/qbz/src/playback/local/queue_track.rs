//! Building a `QueueTrack` from a local-library row, plus the on-disk
//! cover-art backfill run before tracks are queued.

use qbz_models::QueueTrack;

/// Build a `QueueTrack` from a local-library row. Mirrors Tauri's
/// `local_track_to_queue_track`: `file://` artwork, kHz sample rate, the real
/// source. Offline copies carry the Qobuz id (so the shared resolver finds
/// them) + `source = "qobuz_download"`; user files carry the library row id +
/// `source = "local"`.
pub(crate) fn local_queue_track(track: &qbz_library::LocalTrack) -> QueueTrack {
    // Source-aware: offline copies read as Qobuz downloads (carry the Qobuz id
    // so the shared resolver finds them); ephemeral tracks keep their synthetic
    // high id + an "ephemeral" tag so playback routes to the in-memory store;
    // everything else is a real local user file.
    let src = match track.source.as_deref() {
        Some("qobuz_download") => "qobuz_download",
        Some("ephemeral") => "ephemeral",
        _ => "local",
    };
    let is_offline = src == "qobuz_download";
    let artwork_url = track.artwork_path.as_ref().map(|p| {
        if p.starts_with("file://") {
            p.clone()
        } else {
            format!("file://{p}")
        }
    });
    let sample_rate_khz = if track.sample_rate >= 1000.0 {
        track.sample_rate / 1000.0
    } else {
        track.sample_rate
    };
    QueueTrack {
        id: if is_offline {
            track.qobuz_track_id.unwrap_or(track.id) as u64
        } else {
            track.id as u64
        },
        title: track.title.clone(),
        version: None,
        artist: track.artist.clone(),
        album: track.album_group_title.clone(),
        // Local tracks have no Qobuz album-version concept.
        album_version: None,
        duration_secs: track.duration_secs,
        artwork_url,
        hires: track.bit_depth.map(|d| d > 16).unwrap_or(false),
        bit_depth: track.bit_depth,
        sample_rate: Some(sample_rate_khz),
        is_local: true,
        // album_id is the navigation key (now-playing "go to album", Recently
        // Played, record_recent). Local files: the group key is already the
        // right navigation key.
        album_id: Some(track.album_group_key.clone()),
        artist_id: None,
        streamable: true,
        source: Some(src.to_string()),
        parental_warning: false,
        source_item_id_hint: if is_offline {
            // Offline copies: carry the local-library row id. The queue `id`
            // above is the Qobuz catalog id (so the shared resolver finds the
            // track), but every local surface's track row binds the DB row id —
            // NowPlayingState re-publishes it as local-track-id so active-row
            // comparisons (TrackPlayCell/TrackRow/KioskAlbum) match either.
            Some(track.id.to_string())
        } else {
            None
        },
        // Container origin is stamped by the play path (stamp_queue_context);
        // the generic builder leaves it unset.
        context_kind: None,
        context_id: None,
    }
}

/// Fill `artwork_path` for tracks that lack one, from a cover image in the
/// track's folder (the offline-cache writes `cover.jpg` there but doesn't
/// always backfill the index) — so the cover that exists on disk reaches the
/// now-playing bar + queue, not just the album grid. Runs off-thread (fs),
/// memoized per folder so a whole album costs one stat.
pub fn fill_missing_covers(tracks: &mut [qbz_library::LocalTrack]) {
    use std::collections::HashMap;
    let mut memo: HashMap<String, Option<String>> = HashMap::new();
    for t in tracks.iter_mut() {
        if t.artwork_path.as_deref().is_some_and(|s| !s.is_empty()) {
            continue;
        }
        let p = std::path::Path::new(&t.file_path);
        let folder = if p.is_dir() {
            p.to_path_buf()
        } else {
            match p.parent() {
                Some(d) => d.to_path_buf(),
                None => continue,
            }
        };
        let key = folder.to_string_lossy().into_owned();
        let cover = memo
            .entry(key)
            // Robust on-disk cover lookup (cover/folder/front/art/<album>.jpg,
            // any image as a last resort) — shared with the Folders subcards.
            .or_insert_with(|| crate::local_library::find_folder_cover(&folder))
            .clone();
        if cover.is_some() {
            t.artwork_path = cover;
        }
    }
}
