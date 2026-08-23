use qbz_models::{QueueTrack, RepeatMode, Track};

pub(super) fn repeat_str(mode: RepeatMode) -> String {
    match mode {
        RepeatMode::Off => "off",
        RepeatMode::All => "all",
        RepeatMode::One => "one",
    }
    .to_string()
}

/// Qobuz catalog `Track` (`crates/qbz-models/src/types.rs:301-342`, what
/// `core.get_track` returns) -> `QueueTrack` (`crates/qbz-models/src/
/// playback.rs:15`, what the queue stores). An independent Slint-free
/// re-derivation of the same mapping the desktop's single-track play path
/// performs (`crates/qbz/src/playback.rs:2028-2073`) and `qbz-mixtape`'s
/// `track_to_queue_track_from_api` (`crates/qbz-mixtape/src/enqueue.rs:
/// 430-472`) — `qbzd` cannot depend on `qbz` (Slint) and this task's file
/// list does not add a new workspace dependency, so the ~20-line mapping is
/// duplicated rather than imported. `source_item_id_hint`/`context_kind`/
/// `context_id` are left `None`: those are "playing from" provenance fields
/// with no equivalent in a bare `qbzd queue add <TRACK_ID>` call (no album/
/// playlist/artist container in play).
pub(crate) fn track_to_queue_track(track: &Track) -> QueueTrack {
    let artwork_url = track.album.as_ref().and_then(|a| a.image.best().cloned());
    let artist = track
        .performer
        .as_ref()
        .map(|p| p.name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Unknown Artist".to_string());
    let album = track
        .album
        .as_ref()
        .map(|a| a.title.clone())
        .unwrap_or_else(|| "Unknown Album".to_string());
    let album_id = track.album.as_ref().map(|a| a.id.clone());
    let artist_id = track.performer.as_ref().map(|p| p.id);

    QueueTrack {
        id: track.id,
        title: track.title.clone(),
        version: track.version.clone(),
        artist,
        album,
        album_version: None,
        duration_secs: track.duration as u64,
        artwork_url,
        hires: track.hires,
        bit_depth: track.maximum_bit_depth,
        sample_rate: track.maximum_sampling_rate,
        is_local: false,
        album_id,
        artist_id,
        streamable: track.streamable,
        source: Some("qobuz".to_string()),
        parental_warning: track.parental_warning,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}
