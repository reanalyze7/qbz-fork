use qbz_media_controls::MediaEvent;
use tokio::runtime::Handle;

use crate::paths::ProfileRoots;

use super::Runtime;

/// Map one inbound `MediaEvent` onto a core transport command. Runs on the
/// mpris-server (D-Bus) thread — NOT a tokio worker — so the sync core commands
/// are called directly; the async advance ritual is spawned fire-and-forget so
/// the D-Bus thread never blocks on a network resolve. Time values are micros.
pub(super) fn handle_media_event(rt: &Runtime, roots: &ProfileRoots, handle: &Handle, ev: MediaEvent) {
    let core = rt.core();
    match ev {
        MediaEvent::Play => {
            let _ = core.resume();
        }
        MediaEvent::Pause => {
            let _ = core.pause();
        }
        MediaEvent::Toggle => {
            let player = core.player();
            if player.get_playback_event().is_playing {
                let _ = core.pause();
            } else if player.has_loaded_audio() {
                let _ = core.resume();
            }
        }
        MediaEvent::Stop => {
            let _ = core.stop();
        }
        MediaEvent::Next => spawn_advance(rt, roots, handle, true),
        MediaEvent::Previous => spawn_advance(rt, roots, handle, false),
        MediaEvent::SeekBy(micros) => {
            let player = core.player();
            if player.is_dsd_direct_active() {
                return;
            }
            let ev = player.get_playback_event();
            let target = (ev.position as i64 + micros / 1_000_000).max(0) as u64;
            let clamped = if ev.duration > 0 { target.min(ev.duration) } else { target };
            let _ = core.seek(clamped);
        }
        MediaEvent::SetPosition(micros) => {
            let player = core.player();
            if player.is_dsd_direct_active() {
                return;
            }
            let ev = player.get_playback_event();
            let target = (micros.max(0) as u64) / 1_000_000;
            let clamped = if ev.duration > 0 { target.min(ev.duration) } else { target };
            let _ = core.seek(clamped);
        }
        MediaEvent::SetVolume(vol) => {
            let player = core.player();
            if !player.is_dsd_direct_active() {
                let _ = core.set_volume((vol as f32).clamp(0.0, 1.0));
            }
        }
        // Headless daemon: no window to raise, and self-quit on a media-widget
        // "close" would be surprising — ignore both.
        MediaEvent::Raise | MediaEvent::Quit => {}
    }
}

/// Fire-and-forget the FULL advance ritual (skip-walk → play → prefetch →
/// persist) off the D-Bus thread, at the daemon's persisted streaming quality
/// (the same key the driver seeds at boot).
fn spawn_advance(rt: &Runtime, roots: &ProfileRoots, handle: &Handle, forward: bool) {
    let rt = rt.clone();
    let quality = qbz_app::playback_driver::quality_from_key(
        &qbz_app::settings::daemon_prefs::load_at(&roots.data).streaming_quality,
    );
    handle.spawn(async move {
        let _ = qbz_app::playback_driver::advance_and_play(rt.as_ref(), quality, forward).await;
    });
}
