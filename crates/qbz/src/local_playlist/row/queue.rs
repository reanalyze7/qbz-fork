use qbz_models::QueueTrack;

use super::RowItem;

/// Build the queue track for a resolved row, if it is playable.
pub(crate) fn row_queue_track(item: &RowItem) -> Option<QueueTrack> {
    match item {
        RowItem::Qobuz(track) => {
            let (album_id, album_title, album_artwork) = track
                .album
                .as_ref()
                .map(|a| {
                    (
                        a.id.clone(),
                        a.title.clone(),
                        a.image.best().cloned().unwrap_or_default(),
                    )
                })
                .unwrap_or_default();
            let album_artist = track
                .performer
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_default();
            Some(crate::playback::make_queue_track(
                track,
                &album_id,
                &album_title,
                &album_artist,
                &album_artwork,
                None,
            ))
        }
        RowItem::Cached {
            track_id,
            title,
            artist,
            album,
            duration_secs,
            bit_depth,
            sample_rate,
            ..
        } => Some(QueueTrack {
            id: *track_id,
            title: title.clone(),
            version: None,
            artist: artist.clone(),
            album: album.clone(),
            album_version: None,
            duration_secs: *duration_secs,
            artwork_url: None,
            hires: bit_depth.map(|d| d >= 24).unwrap_or(false),
            bit_depth: *bit_depth,
            sample_rate: *sample_rate,
            is_local: false,
            album_id: None,
            artist_id: None,
            streamable: true,
            // Plain "qobuz": the play tier-walk serves the offline-cache hit.
            source: Some("qobuz".to_string()),
            parental_warning: false,
            source_item_id_hint: None,
            context_kind: None,
            context_id: None,
        }),
        RowItem::Local(track) => {
            Some(crate::playback::local_queue_track(track))
        }
        // Filename-fallback rows have no library row to resolve playback
        // through — render-only until the file is re-indexed.
        RowItem::LocalFile { .. } => None,
        RowItem::Unresolved { .. } => None,
    }
}
