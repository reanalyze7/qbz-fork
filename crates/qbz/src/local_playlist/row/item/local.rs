use qbz_models::QueueTrack;

use crate::local_playlist::row::mmss;
use crate::TrackItem;

pub(super) fn local_item(track: &qbz_library::LocalTrack, queue: Option<&QueueTrack>) -> TrackItem {
    let (tier, quality_detail, _) = crate::quality::badge(
        &track.format.to_string(),
        track.bit_depth,
        Some(track.sample_rate),
    );
    TrackItem {
        // Local / offline rows are protected — never blacklisted.
        is_blacklisted: false,
        // The queue id (library row id; the Qobuz id for offline
        // copies) so visible-order playback resolves this row.
        id: queue
            .map(|q| q.id.to_string())
            .unwrap_or_else(|| track.id.to_string())
            .into(),
        number: "".into(),
        title: track.title.clone().into(),
        artist: track.artist.clone().into(),
        album: track.album.clone().into(),
        duration: mmss(track.duration_secs).into(),
        quality_tier: tier.into(),
        quality_detail: quality_detail.into(),
        explicit: false,
        selected: false,
        artwork_url: track.artwork_path.clone().unwrap_or_default().into(),
        artwork: slint::Image::default(),
        is_favorite: false,
        artist_id: "".into(),
        album_id: "".into(),
        removing: false,
        cache_status: 0,
        cache_progress: 0.0,
        source: match track.source.as_deref() {
            Some("qobuz_download") => "qobuz",
            _ => "local",
        }
        .into(),
        unlocking: false,
        // Disc grouping is album-detail only; playlist rows carry none.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}
