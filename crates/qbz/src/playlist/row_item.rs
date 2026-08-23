//! `Track` -> `TrackItem` mapper — the online-playlist row converter,
//! pulling in blacklist/favorite/offline-cache/quality state per row.

use qbz_models::Track;

use crate::TrackItem;

fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub(crate) fn to_item(track: &Track) -> TrackItem {
    let mut title = track.title.clone();
    if let Some(v) = track.version.as_ref().filter(|v| !v.is_empty()) {
        title = format!("{title} ({v})");
    }
    // Blacklist key: the track's performer OR composer id (pure-Qobuz playlist
    // rows; local rows go through local_playlist::row_item, never
    // stamped). Composer included so the row greyout matches the queue
    // predicate (D-FEAT: performer OR composer).
    let performer_id = track
        .performer
        .as_ref()
        .map(|p| p.id.to_string())
        .unwrap_or_default();
    let composer_id = track
        .composer
        .as_ref()
        .map(|c| c.id.to_string())
        .unwrap_or_default();
    TrackItem {
        is_blacklisted: crate::artist_blacklist::stamp_row(
            "qobuz",
            &[performer_id.as_str(), composer_id.as_str()],
            track.album.as_ref().map(|a| a.id.as_str()),
        ),
        id: track.id.to_string().into(),
        number: "".into(),
        title: title.into(),
        artist: track
            .performer
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default()
            .into(),
        album: track
            .album
            .as_ref()
            .map(|a| a.title.clone())
            .unwrap_or_default()
            .into(),
        duration: mmss(track.duration).into(),
        quality_tier: match track.maximum_bit_depth {
            Some(d) if d >= 24 => "hires",
            Some(_) => "cd",
            None => "",
        }
        .into(),
        quality_detail: crate::quality::detail(
            track.maximum_bit_depth,
            track.maximum_sampling_rate,
        )
        .into(),
        explicit: track.parental_warning,
        selected: false,
        // Smallest variant — these are 40px row thumbnails; best()
        // would download mega/large covers (2000-row perf killer).
        artwork_url: track
            .album
            .as_ref()
            .and_then(|a| a.image.smallest().cloned())
            .unwrap_or_default()
            .into(),
        artwork: slint::Image::default(),
        is_favorite: crate::fav_cache::is_favorite(&track.id.to_string()),
        artist_id: track
            .performer
            .as_ref()
            .map(|p| p.id.to_string())
            .unwrap_or_default()
            .into(),
        album_id: track
            .album
            .as_ref()
            .map(|a| a.id.clone())
            .unwrap_or_default()
            .into(),
        removing: false,
        cache_status: if crate::offline_cache::is_cached(&track.id.to_string()) { 3 } else { 0 },
        cache_progress: 0.0,
        source: "qobuz".into(),
        unlocking: false,
        // Disc grouping is album-detail only; flat lists carry none.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}
