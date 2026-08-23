//! `TrayHandle` — the cross-platform live-update handle.

#[cfg(target_os = "linux")]
use super::linux;
#[cfg(target_os = "macos")]
use super::macos;

/// Cross-thread handle to the live tray. Cloneable; mutators forward to the
/// platform backend (ksni on Linux) and are no-ops when the tray is disabled
/// or on a platform without a live-update path.
#[derive(Clone, Default)]
pub struct TrayHandle {
    #[cfg(target_os = "linux")]
    pub(super) linux: Option<linux::LinuxTrayHandle>,
}

impl TrayHandle {
    pub fn set_track(&self, title: String, artist: String, album: String) {
        #[cfg(target_os = "linux")]
        if let Some(h) = &self.linux {
            h.set_track(title, artist, album);
            return;
        }
        #[cfg(not(target_os = "linux"))]
        let _ = (title, artist, album);
    }

    pub fn clear_track(&self) {
        #[cfg(target_os = "linux")]
        if let Some(h) = &self.linux {
            h.clear_track();
        }
    }

    pub fn set_playing(&self, is_playing: bool) {
        #[cfg(target_os = "linux")]
        if let Some(h) = &self.linux {
            h.set_playing(is_playing);
            return;
        }
        #[cfg(not(target_os = "linux"))]
        let _ = is_playing;
    }

    pub fn set_icon_theme(&self, theme: String) {
        #[cfg(target_os = "linux")]
        if let Some(h) = &self.linux {
            h.set_icon_theme(theme);
            return;
        }
        #[cfg(target_os = "macos")]
        {
            // The NSStatusItem is !Send and lives on the main thread — re-theme
            // it there.
            let _ = slint::invoke_from_event_loop(move || macos::set_icon_theme(&theme));
            return;
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        let _ = theme;
    }
}
