use std::path::Path;

use crate::{AudioFormat, AudioProperties, LocalTrack, MetadataExtractor};

use super::model::CueSheet;

/// Convert a CUE sheet into LocalTrack entries
pub fn cue_to_tracks(
    cue: &CueSheet,
    audio_duration_secs: u64,
    format: AudioFormat,
    properties: &AudioProperties,
) -> Vec<LocalTrack> {
    let mut tracks = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let cue_audio_path = Path::new(&cue.audio_file);
    let (album_group_key, album_group_title) =
        MetadataExtractor::album_group_info(cue_audio_path, cue.title.as_deref());
    let inferred_disc = MetadataExtractor::infer_disc_number(cue_audio_path);

    for (i, cue_track) in cue.tracks.iter().enumerate() {
        // Calculate end time (next track's start or audio end)
        let end_secs = if i + 1 < cue.tracks.len() {
            cue.tracks[i + 1].start_secs
        } else {
            audio_duration_secs as f64
        };

        let duration = (end_secs - cue_track.start_secs).max(0.0) as u64;

        tracks.push(LocalTrack {
            id: 0,
            file_path: cue.audio_file.clone(),
            title: cue_track.title.clone(),
            artist: cue_track
                .performer
                .clone()
                .or_else(|| cue.performer.clone())
                .unwrap_or_else(|| "Unknown Artist".to_string()),
            album: cue
                .title
                .clone()
                .unwrap_or_else(|| "Unknown Album".to_string()),
            album_artist: cue.performer.clone(),
            album_group_key: album_group_key.clone(),
            album_group_title: album_group_title.clone(),
            track_number: Some(cue_track.number),
            disc_number: inferred_disc,
            year: None,
            genre: None,
            catalog_number: None,
            duration_secs: duration,
            format: format.clone(),
            bit_depth: properties.bit_depth,
            sample_rate: properties.sample_rate,
            channels: properties.channels,
            file_size_bytes: 0,
            cue_file_path: Some(cue.file_path.clone()),
            cue_start_secs: Some(cue_track.start_secs),
            cue_end_secs: Some(end_secs),
            artwork_path: None,
            last_modified: 0,
            indexed_at: now,
            source: None,
            qobuz_track_id: None,
            is_network_mount: false,
        });
    }

    tracks
}
