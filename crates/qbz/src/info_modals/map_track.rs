//! `Track` → plain-struct mapping (worker thread).

use qbz_models::Track;
use qbz_qobuz::performers::{format_role_label, group_credits_ordered, parse_performers};

use super::format::{format_title, track_duration, track_quality};
use super::types::{CreditRowData, TrackInfoData};

pub(super) fn map_track_info(track: Track) -> TrackInfoData {
    let title = format_title(&track.title, track.version.as_deref());

    let (artist, artist_id) = match track.performer.as_ref() {
        Some(a) if a.id != 0 => (a.name.clone(), a.id.to_string()),
        Some(a) => (a.name.clone(), String::new()),
        None => (String::new(), String::new()),
    };

    let album_title = track
        .album
        .as_ref()
        .map(|a| a.title.clone())
        .unwrap_or_default();

    let (label, label_id) = match track.album.as_ref().and_then(|a| a.label.as_ref()) {
        Some(l) => (l.name.clone(), l.id.to_string()),
        None => (String::new(), String::new()),
    };

    let credits = group_credits_ordered(&parse_performers(
        track.performers.as_deref().unwrap_or_default(),
    ))
    .into_iter()
    .map(|(role, names)| CreditRowData {
        role: format_role_label(&role).to_uppercase(),
        role_raw: role,
        names,
    })
    .collect();

    TrackInfoData {
        title,
        album: album_title,
        artist,
        artist_id,
        duration: track_duration(track.duration),
        quality: track_quality(
            track.maximum_bit_depth,
            track.maximum_sampling_rate,
            track.hires_streamable,
        ),
        isrc: track.isrc.unwrap_or_default(),
        label,
        label_id,
        copyright: track
            .copyright
            .as_deref()
            .map(crate::strip_html::decode_html_entities)
            .unwrap_or_default(),
        credits,
    }
}
