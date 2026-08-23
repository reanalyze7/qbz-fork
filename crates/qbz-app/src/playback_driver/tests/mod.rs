mod cadence_and_errors;
mod edges;

use qbz_player::PlaybackEvent;

use crate::playback_driver::QueueSnapshot;

/// Minimal `PlaybackEvent` builder: the four fields the driver reasons about
/// plus Default-equivalents for the rest (gapless fields off, no stream meta).
pub(super) fn ev(track: u64, playing: bool, pos: u64, dur: u64) -> PlaybackEvent {
    PlaybackEvent {
        is_playing: playing,
        position: pos,
        duration: dur,
        track_id: track,
        volume: 1.0,
        sample_rate: None,
        bit_depth: None,
        shuffle: None,
        repeat: None,
        normalization_gain: None,
        gapless_ready: false,
        gapless_next_track_id: 0,
        bit_perfect_mode: None,
        buffer_progress: None,
    }
}

/// Queue-shape builder: current track id, `(id, streamable)` upcoming list,
/// repeat key ("off"|"all"|"one"), optional stop-after marker.
pub(super) fn q(
    current: u64,
    upcoming: &[(u64, bool)],
    repeat: &str,
    stop_after: Option<u64>,
) -> QueueSnapshot {
    QueueSnapshot {
        current,
        upcoming: upcoming.to_vec(),
        repeat: repeat.to_string(),
        stop_after,
        autoplay_infinite: false,
    }
}
