use std::time::Duration;

use qbz_media_controls::{PlaybackStatus, TrackMeta};
use qbz_models::{PlaybackState, QueueTrack};

pub(super) fn track_meta(t: &QueueTrack) -> TrackMeta {
    TrackMeta {
        title: t.title.clone(),
        artist: t.artist.clone(),
        album: t.album.clone(),
        duration: (t.duration_secs > 0).then(|| Duration::from_secs(t.duration_secs)),
        art_url: t.artwork_url.clone(),
    }
}

pub(super) fn map_state(s: PlaybackState) -> PlaybackStatus {
    match s {
        PlaybackState::Playing => PlaybackStatus::Playing,
        PlaybackState::Paused => PlaybackStatus::Paused,
        PlaybackState::Stopped => PlaybackStatus::Stopped,
        // Buffering is still "playing" from the user's point of view.
        PlaybackState::Loading => PlaybackStatus::Playing,
    }
}
