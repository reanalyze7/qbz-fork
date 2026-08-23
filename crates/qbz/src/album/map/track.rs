//! `Track` -> `TrackData` mapper + its small quality/duration helpers.

use qbz_models::Track;

use super::super::data::TrackData;
use super::text::format_duration;

pub(super) fn map_track(track: Track) -> TrackData {
    // Classical work metadata, read before `title`/`performer` are moved out of
    // `track`. Qobuz serves `work` on the track (null for non-classical) and a
    // `composer` artist; the official player renders the work title with the
    // composer parenthesized AND the composer name is a link to the artist page
    // (PR #536 + E3). `work` holds the TITLE only (for run-length grouping); the
    // composer name + id are carried separately so the view can make the name a
    // clickable link. All "" when there is no work.
    let work = track
        .work
        .as_ref()
        .filter(|w| !w.is_empty())
        .cloned()
        .unwrap_or_default();
    let (work_composer_name, work_composer_id) = if work.is_empty() {
        (String::new(), String::new())
    } else {
        track
            .composer
            .as_ref()
            .filter(|c| !c.name.is_empty())
            .map(|c| (c.name.clone(), c.id.to_string()))
            .unwrap_or_default()
    };
    let mut title = track.title;
    if let Some(version) = track.version.as_ref().filter(|v| !v.is_empty()) {
        title = format!("{title} ({version})");
    }
    let (artist, artist_id) = track
        .performer
        .map(|p| (p.name, p.id.to_string()))
        .unwrap_or_default();
    TrackData {
        id: track.id.to_string(),
        number: track.track_number.to_string(),
        title,
        artist,
        artist_id,
        // The album view stamps the viewed album's id at the apply layer.
        album_id: String::new(),
        // Album title + cover are stamped by the apply layer (album view rows
        // all belong to the viewed album); leave empty here.
        album: String::new(),
        artwork_url: String::new(),
        duration: mmss(track.duration),
        quality_tier: tier(track.maximum_bit_depth).to_string(),
        quality_detail: crate::quality::detail(
            track.maximum_bit_depth,
            track.maximum_sampling_rate,
        ),
        explicit: track.parental_warning,
        // Tauri: `disc = track.media_number ?? 1`.
        disc: track.media_number.unwrap_or(1),
        work,
        work_composer_name,
        work_composer_id,
    }
}

/// 24-bit and up is Hi-Res, anything else with depth info is CD-quality.
pub(in crate::album) fn tier(bit_depth: Option<u32>) -> &'static str {
    match bit_depth {
        Some(depth) if depth >= 24 => "hires",
        Some(_) => "cd",
        None => "",
    }
}

/// `m:ss` track duration.
pub(in crate::album) fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}
