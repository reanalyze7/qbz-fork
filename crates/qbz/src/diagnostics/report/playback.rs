//! The `## Playback` markdown section.

use super::super::rows::{opt, trim_khz, yn};
use super::md_line;

pub(super) fn write_section(
    out: &mut String,
    pb: &qbz_player::PlaybackState,
    track: Option<&qbz_models::QueueTrack>,
) {
    out.push_str("\n## Playback\n\n");
    let volume_percent = (pb.volume * 100.0).round() as i64;
    let title = track.map(|t| t.title.clone());
    let artist = track.map(|t| t.artist.clone());
    let album = track.map(|t| t.album.clone());
    let source = track.and_then(|t| t.source.clone());
    let bit_depth = track
        .and_then(|t| t.bit_depth)
        .map(|b| format!("{b}-bit"))
        .unwrap_or_else(|| "—".to_string());
    let track_sample_rate = track
        .and_then(|t| t.sample_rate)
        .map(|r| format!("{} kHz", trim_khz(r)))
        .unwrap_or_else(|| "—".to_string());
    let is_local = match track {
        Some(t) => yn(t.is_local).to_string(),
        None => "—".to_string(),
    };
    md_line(out, "Playing", yn(pb.is_playing));
    md_line(out, "Volume", &format!("{volume_percent}%"));
    md_line(
        out,
        "Position / Duration",
        &format!("{}s / {}s", pb.position, pb.duration),
    );
    md_line(out, "Has Track", yn(track.is_some()));
    md_line(out, "Track Title", &opt(&title));
    md_line(out, "Track Artist", &opt(&artist));
    md_line(out, "Track Album", &opt(&album));
    md_line(out, "Track Source", &opt(&source));
    md_line(out, "Track Is Local", &is_local);
    md_line(out, "Track Quality", "—");
    md_line(out, "Track Format", "—");
    md_line(out, "Track Bit Depth", &bit_depth);
    md_line(out, "Track Sample Rate", &track_sample_rate);
}
