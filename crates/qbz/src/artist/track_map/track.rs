use qbz_models::PageArtistTrack;

use crate::album::TrackData;

pub(crate) fn tier(bit_depth: Option<u32>) -> &'static str {
    match bit_depth {
        Some(depth) if depth >= 24 => "hires",
        Some(_) => "cd",
        None => "",
    }
}

pub(crate) fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub(crate) fn map_track(index: usize, track: PageArtistTrack) -> TrackData {
    let mut title = track.title;
    if let Some(version) = track.version.as_ref().filter(|v| !v.is_empty()) {
        title = format!("{title} ({version})");
    }
    let (artist, artist_id) = track
        .artist
        .map(|a| (a.name.display, a.id.to_string()))
        .unwrap_or_default();
    // Pull the album id, title AND cover from the same nested album object —
    // the /artist/page response already carries all three, so the row can show
    // the album name + thumbnail without an extra request. `smallest()` is the
    // list-row thumbnail variant (best() would download the mega/large cover).
    let (album_id, album, artwork_url) = track
        .album
        .map(|a| {
            let url = a.image.and_then(|img| img.smallest().cloned()).unwrap_or_default();
            (a.id, a.title, url)
        })
        .unwrap_or_default();
    let bit_depth = track.audio_info.as_ref().and_then(|a| a.maximum_bit_depth);
    let sample_rate = track.audio_info.as_ref().and_then(|a| a.maximum_sampling_rate);
    TrackData {
        id: track.id.to_string(),
        number: (index + 1).to_string(),
        title,
        artist,
        artist_id,
        album_id,
        album,
        artwork_url,
        duration: mmss(track.duration.unwrap_or(0)),
        quality_tier: tier(bit_depth).to_string(),
        quality_detail: crate::quality::detail(bit_depth, sample_rate),
        explicit: track.parental_warning.unwrap_or(false),
        // Artist top-tracks are a flat cross-album list and never render
        // "Disc N" headers, so the disc value is unused here — default to 1.
        disc: 1,
        // Work-section headers are album-view only; the flat artist list never
        // renders them, so leave them empty.
        work: String::new(),
        work_composer_name: String::new(),
        work_composer_id: String::new(),
    }
}
