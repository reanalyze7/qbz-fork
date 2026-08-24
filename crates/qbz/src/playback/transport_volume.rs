//! Volume + mute transport controls, split out of `transport.rs` to keep
//! both files under the line budget.
use slint::ComponentHandle;

use super::Runtime;
use crate::{AppWindow, NowPlayingState};

/// Mute state and the volume to restore on unmute. `PREMUTE_VOLUME`
/// holds the f32 level as bits; `MUTED` is the authoritative flag.
static MUTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PREMUTE_VOLUME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Set the player volume from `fraction` (0..1). A non-zero level clears
/// any active mute, so dragging the slider or stepping volume unmutes.
pub fn set_volume(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    fraction: f32,
) {
    handle.spawn(async move {
        let fraction = fraction.clamp(0.0, 1.0);
        if let Err(e) = runtime.core().set_volume(fraction) {
            log::error!("[qbz-slint] playback: set_volume failed: {e}");
        }
        if fraction > 0.0 && MUTED.swap(false, std::sync::atomic::Ordering::Relaxed) {
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NowPlayingState>().set_muted(false);
            });
        }
    });
}

/// Toggle mute: silence the player and remember the level, or restore it.
pub fn toggle_mute(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    use std::sync::atomic::Ordering;
    handle.spawn(async move {
        if MUTED.swap(false, Ordering::Relaxed) {
            // Unmute — restore the stored level.
            let restored = f32::from_bits(PREMUTE_VOLUME.load(Ordering::Relaxed));
            let restored = if restored > 0.0 { restored } else { 0.7 };
            let _ = runtime.core().set_volume(restored);
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NowPlayingState>().set_muted(false);
            });
        } else {
            // Mute — stash the current level, then drop to zero.
            let current = runtime.core().player().get_playback_event().volume;
            let current = if current > 0.0 { current } else { 0.7 };
            PREMUTE_VOLUME.store(current.to_bits(), Ordering::Relaxed);
            MUTED.store(true, Ordering::Relaxed);
            let _ = runtime.core().set_volume(0.0);
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NowPlayingState>().set_muted(true);
            });
        }
    });
}
