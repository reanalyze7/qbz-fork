//! Local -> rendered mapping for the Tracks tab.

use crate::TrackItem;

pub(crate) fn fmt_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Map one local track row to the rendered `TrackItem` (UI thread — holds a
/// non-Send `slint::Image`). Local tracks aren't Qobuz-linkable, so the
/// artist/album link ids are empty (the row renders them as plain text).
pub(crate) fn map_local_track(t: qbz_library::LocalTrack) -> TrackItem {
    // One shared classifier (crate::quality::badge) — same source the album
    // card + header use, so all surfaces agree; an un-hydrated lossless track
    // shows a generic "FLAC" detail. `t.sample_rate` is Hz; badge normalizes.
    let (tier, quality_detail, _) =
        crate::quality::badge(&t.format.to_string(), t.bit_depth, Some(t.sample_rate));
    TrackItem {
        // Local Library rows are local assets — never blacklisted (protected).
        is_blacklisted: false,
        id: t.id.to_string().into(),
        number: t.track_number.map(|n| n.to_string()).unwrap_or_default().into(),
        title: t.title.into(),
        artist: t.artist.into(),
        album: t.album.into(),
        duration: fmt_duration(t.duration_secs).into(),
        quality_tier: tier.into(),
        quality_detail: quality_detail.into(),
        explicit: false,
        selected: false,
        artwork_url: t.artwork_path.unwrap_or_default().into(),
        artwork: slint::Image::default(),
        // Local favorite state from the local-favorites store, keyed by the
        // stable track key (file_path). Offline / ephemeral rows are not
        // locally favoritable.
        is_favorite: match t.source.as_deref() {
            Some("qobuz_download") | Some("ephemeral") => false,
            _ => crate::local_favorites::is_favorite("track", &t.file_path),
        },
        artist_id: "".into(),
        album_id: "".into(),
        removing: false,
        cache_status: 0,
        cache_progress: 0.0,
        // Source indicator: offline copies read as Qobuz, user files as local,
        // ephemeral tracks tagged so the UI can gate persistence actions.
        source: match t.source.as_deref() {
            Some("qobuz_download") => "qobuz",
            Some("ephemeral") => "ephemeral",
            _ => "local",
        }
        .into(),
        unlocking: false,
        // Default: no disc header. The flat Library Tracks tab never groups by
        // disc; the local-album DETAIL view stamps this afterwards (see
        // apply_album_version) for multi-disc local albums.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}
