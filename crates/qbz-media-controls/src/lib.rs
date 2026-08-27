//! Frontend-agnostic system media controls (ADR-006).
//!
//! - **Linux:** `mpris-server` — publishes the full MPRIS interface INCLUDING
//!   `org.mpris.MediaPlayer2.DesktopEntry = "io.github.reanalyze7.qoqobuz"`, the only way
//!   GNOME Shell resolves the application icon for its media widget. (souvlaki,
//!   the cross-platform crate, never sets it, so GNOME shows no app icon.)
//! - **macOS / Windows:** `souvlaki` — MediaRemote / SMTC, where there is no
//!   DesktopEntry concept (macOS keys the icon off the app bundle).
//!
//! One trait ([`MediaIntegration`]); one factory ([`spawn`]); no winit / Slint
//! / Tauri types — headless/TUI can reuse it.

use std::sync::Arc;

mod types;
pub use types::{MediaEvent, MediaIntegration, PlaybackStatus, TrackMeta};

pub mod notify;
pub use notify::{show_track_notification, NotificationMeta};

#[cfg(target_os = "linux")]
mod inhibit;
#[cfg(target_os = "linux")]
mod linux;

/// Spawn the OS media-controls integration (MPRIS). `on_event` receives
/// inbound control events (media keys, the desktop media widget). Returns
/// `None` if the backend could not start — the app keeps working without
/// media controls.
pub fn spawn(
    on_event: impl Fn(MediaEvent) + Send + Sync + 'static,
) -> Option<Box<dyn MediaIntegration>> {
    let cb: Arc<dyn Fn(MediaEvent) + Send + Sync> = Arc::new(on_event);

    #[cfg(target_os = "linux")]
    {
        return linux::spawn(cb).map(|h| Box::new(h) as Box<dyn MediaIntegration>);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cb;
        None
    }
}
