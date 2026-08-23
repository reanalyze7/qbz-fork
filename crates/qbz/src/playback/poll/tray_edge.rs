//! Tray tooltip + OS media-controls play/pause mirror, fired only on the
//! playing/paused transition edge (not every tick).

/// Reflect play/pause into the tray tooltip on transition only (Linux), so
/// the "Middle-click to pause/play" hint stays correct without spamming the
/// updater channel every tick.
pub(super) fn on_transition(is_playing: bool, was_playing: bool, position: u64) {
    if is_playing == was_playing {
        return;
    }
    if let Some(t) = crate::tray::handle() {
        t.set_playing(is_playing);
    }
    if let Some(mc) = crate::media_controls::handle() {
        let status = if is_playing {
            qbz_media_controls::PlaybackStatus::Playing
        } else {
            qbz_media_controls::PlaybackStatus::Paused
        };
        mc.set_playback(status, Some(std::time::Duration::from_secs(position)));
    }
}
