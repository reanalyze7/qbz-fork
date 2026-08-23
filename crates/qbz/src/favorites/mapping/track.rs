use qbz_models::Track;

use crate::album_map;
use crate::favorites::mapping::TrackCard;

pub(crate) fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub(crate) fn map_track(track: Track) -> TrackCard {
    let mut title = track.title;
    if let Some(version) = track.version.as_ref().filter(|v| !v.is_empty()) {
        title = format!("{title} ({version})");
    }
    let artwork_url = track
        .album
        .as_ref()
        .and_then(|a| a.image.best().cloned())
        .unwrap_or_default();
    let album = track
        .album
        .as_ref()
        .map(|a| a.title.clone())
        .unwrap_or_default();
    let album_id = track.album.as_ref().map(|a| a.id.clone()).unwrap_or_default();
    let genre = track
        .album
        .as_ref()
        .and_then(|a| a.genre.as_ref())
        .map(|g| g.name.clone())
        .unwrap_or_default();
    let (artist, artist_id) = track
        .performer
        .map(|p| (p.name, p.id.to_string()))
        .unwrap_or_default();
    // Composer id for the blacklist row stamp (D-FEAT: performer OR composer).
    let composer_id = track
        .composer
        .map(|c| c.id.to_string())
        .unwrap_or_default();
    TrackCard {
        id: track.id.to_string(),
        title,
        artist,
        artist_id,
        composer_id,
        album,
        album_id,
        genre,
        duration: mmss(track.duration),
        quality_tier: album_map::tier(track.maximum_bit_depth).to_string(),
        quality_detail: crate::quality::detail(
            track.maximum_bit_depth,
            track.maximum_sampling_rate,
        ),
        explicit: track.parental_warning,
        artwork_url,
        label_id: track
            .album
            .as_ref()
            .and_then(|a| a.label.as_ref())
            .map(|l| l.id.to_string())
            .unwrap_or_default(),
    }
}
