//! Public mutators on [`LinuxTrayHandle`] — thin wrappers over `send`.

use super::updater::{LinuxTrayHandle, TrayUpdate};

impl LinuxTrayHandle {
    pub fn set_track(&self, title: String, artist: String, album: String) {
        self.send(TrayUpdate::SetTrack {
            title,
            artist,
            album,
        });
    }

    pub fn clear_track(&self) {
        self.send(TrayUpdate::ClearTrack);
    }

    pub fn set_playing(&self, is_playing: bool) {
        self.send(TrayUpdate::SetPlaying(is_playing));
    }

    pub fn set_icon_theme(&self, theme: String) {
        self.send(TrayUpdate::SetIconTheme(theme));
    }
}
