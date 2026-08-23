use crate::local_playlist::row::mmss;
use crate::TrackItem;

#[allow(clippy::too_many_arguments)]
pub(super) fn cached_item(
    track_id: u64,
    title: &str,
    artist: &str,
    album: &str,
    duration_secs: u64,
    bit_depth: Option<u32>,
    sample_rate: Option<f64>,
    artwork_path: &Option<String>,
) -> TrackItem {
    TrackItem {
        // Offline cached copy — a local asset, never blacklisted (no
        // artist id carried; protected by the local guard either way).
        is_blacklisted: false,
        id: track_id.to_string().into(),
        number: "".into(),
        title: title.to_string().into(),
        artist: artist.to_string().into(),
        album: album.to_string().into(),
        duration: mmss(duration_secs).into(),
        quality_tier: match bit_depth {
            Some(d) if d >= 24 => "hires",
            Some(_) => "cd",
            None => "",
        }
        .into(),
        quality_detail: crate::quality::detail(bit_depth, sample_rate).into(),
        explicit: false,
        selected: false,
        artwork_url: artwork_path.clone().unwrap_or_default().into(),
        artwork: slint::Image::default(),
        is_favorite: crate::fav_cache::is_favorite(&track_id.to_string()),
        artist_id: "".into(),
        album_id: "".into(),
        removing: false,
        cache_status: 3,
        cache_progress: 0.0,
        source: "qobuz".into(),
        unlocking: false,
        // Disc grouping is album-detail only; playlist rows carry none.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}
