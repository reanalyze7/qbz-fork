//! Conversions between `QueueTrack` / `RepeatMode` and their persisted
//! schema counterparts.

use qbz_app::session_store::PersistedQueueTrack;
use qbz_models::{QueueTrack, RepeatMode};

pub(super) fn repeat_to_str(mode: RepeatMode) -> &'static str {
    match mode {
        RepeatMode::Off => "off",
        RepeatMode::All => "all",
        RepeatMode::One => "one",
    }
}

pub(super) fn repeat_from_str(s: &str) -> RepeatMode {
    match s {
        "all" => RepeatMode::All,
        "one" => RepeatMode::One,
        _ => RepeatMode::Off,
    }
}

pub(super) fn to_persisted(t: &QueueTrack) -> PersistedQueueTrack {
    PersistedQueueTrack {
        id: t.id,
        title: t.title.clone(),
        artist: t.artist.clone(),
        album: t.album.clone(),
        duration_secs: t.duration_secs,
        artwork_url: t.artwork_url.clone(),
        hires: t.hires,
        bit_depth: t.bit_depth,
        sample_rate: t.sample_rate,
        is_local: t.is_local,
        album_id: t.album_id.clone(),
        artist_id: t.artist_id,
        streamable: t.streamable,
        source: t.source.clone(),
        parental_warning: t.parental_warning,
        source_item_id_hint: t.source_item_id_hint.clone(),
    }
}

pub(super) fn from_persisted(t: PersistedQueueTrack) -> QueueTrack {
    QueueTrack {
        id: t.id,
        title: t.title,
        // The persisted schema predates `version` (Tauri parity): the edition
        // subtitle is not stored, so a restored track simply has no version.
        version: None,
        artist: t.artist,
        album: t.album,
        // Album-version is cosmetic (now-playing/MPRIS); not persisted in the
        // session schema, so a restored track shows the clean album until the
        // next album-play repopulates it.
        album_version: None,
        duration_secs: t.duration_secs,
        artwork_url: t.artwork_url,
        hires: t.hires,
        bit_depth: t.bit_depth,
        sample_rate: t.sample_rate,
        is_local: t.is_local,
        album_id: t.album_id,
        artist_id: t.artist_id,
        streamable: t.streamable,
        source: t.source,
        parental_warning: t.parental_warning,
        source_item_id_hint: t.source_item_id_hint,
        // Not persisted in the session schema — a restored track carries no
        // container origin, so the "playing from" button falls back to the
        // track's own album until the next container play re-stamps the queue.
        context_kind: None,
        context_id: None,
    }
}
