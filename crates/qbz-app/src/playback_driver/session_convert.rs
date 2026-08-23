use qbz_models::QueueTrack;

use crate::session_store::PersistedQueueTrack;

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
        version: None,
        artist: t.artist,
        album: t.album,
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
        context_kind: None,
        context_id: None,
    }
}
