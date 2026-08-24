use super::helpers::{opt, row, trim_khz, yn};

pub(crate) fn build_playback_rows(
    pb: &qbz_player::PlaybackState,
    track: Option<&qbz_models::QueueTrack>,
) -> Vec<crate::DiagRow> {
    let volume_percent = (pb.volume * 100.0).round() as i64;
    let has_track = track.is_some();
    let title = track.map(|t| t.title.clone());
    let artist = track.map(|t| t.artist.clone());
    let album = track.map(|t| t.album.clone());
    let source = track.and_then(|t| t.source.clone());
    let bit_depth = track
        .and_then(|t| t.bit_depth)
        .map(|d| format!("{d}-bit"))
        .unwrap_or_else(|| "—".to_string());
    let sample_rate = track
        .and_then(|t| t.sample_rate)
        .map(|r| format!("{} kHz", trim_khz(r)))
        .unwrap_or_else(|| "—".to_string());
    let is_local = match track {
        Some(t) => yn(t.is_local).to_string(),
        None => "—".to_string(),
    };

    vec![
        row("Playing", "—", yn(pb.is_playing), 0),
        row("Volume", "—", &format!("{volume_percent}%"), 0),
        row(
            "Position / Duration",
            "—",
            &format!("{}s / {}s", pb.position, pb.duration),
            0,
        ),
        row("Has Track", "—", yn(has_track), 0),
        row("Track Title", "—", &opt(&title), 0),
        row("Track Artist", "—", &opt(&artist), 0),
        row("Track Album", "—", &opt(&album), 0),
        row("Track Source", "—", &opt(&source), 0),
        row("Track Is Local", "—", &is_local, 0),
        // No quality/format field on QueueTrack — emit "—" (faithful to data).
        row("Track Quality", "—", "—", 0),
        row("Track Format", "—", "—", 0),
        row("Track Bit Depth", "—", &bit_depth, 0),
        row("Track Sample Rate", "—", &sample_rate, 0),
    ]
}
