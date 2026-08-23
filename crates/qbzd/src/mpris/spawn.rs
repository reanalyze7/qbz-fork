use std::sync::{Arc, Weak};
use std::time::Duration;

use qbz_app::shell::AppRuntime;
use qbz_media_controls::{MediaIntegration, PlaybackStatus};
use qbz_models::CoreEvent;
use tokio::runtime::Handle;
use tokio::sync::broadcast;

use crate::adapter::DaemonAdapter;
use crate::paths::ProfileRoots;

use super::inbound::handle_media_event;
use super::mapping::{map_state, track_meta};
use super::{enabled, MprisHandle, Runtime};

/// Spawn the MPRIS integration. Returns None when disabled, on a non-Linux
/// platform, or when the D-Bus backend can't start (headless server).
pub fn spawn(
    runtime: &Runtime,
    roots: ProfileRoots,
    mut bus: broadcast::Receiver<CoreEvent>,
    handle: Handle,
) -> Option<MprisHandle> {
    if !enabled(&roots) {
        log::info!("[mpris] disabled (playback.mpris / QBZD_MPRIS)");
        return None;
    }

    // INBOUND: media keys / desktop widget → core transport. Weak so the OS
    // integration never keeps the runtime alive.
    let weak: Weak<AppRuntime<DaemonAdapter>> = Arc::downgrade(runtime);
    let cb_handle = handle.clone();
    let integration: Arc<dyn MediaIntegration> = Arc::from(qbz_media_controls::spawn(move |ev| {
        if let Some(rt) = weak.upgrade() {
            handle_media_event(&rt, &roots, &cb_handle, ev);
        }
    })?);
    log::info!("[mpris] publishing org.mpris.MediaPlayer2 (desktop media controls + media keys)");

    // OUTBOUND: seed once from live state, then follow the bus.
    let seed_weak: Weak<AppRuntime<DaemonAdapter>> = Arc::downgrade(runtime);
    let updater_integ = integration.clone();
    let updater = handle.spawn(async move {
        use broadcast::error::RecvError;
        let mut last = PlaybackStatus::Stopped;

        // One-time seed so the widget isn't blank until the next event. The
        // strong Arc is dropped before the loop, keeping the task Weak-only.
        if let Some(rt) = seed_weak.upgrade() {
            let queue = rt.core().get_queue_state().await;
            if let Some(track) = queue.current_track.as_ref() {
                updater_integ.set_metadata(&track_meta(track));
            }
            let player = rt.core().player();
            let ev = player.get_playback_event();
            last = if ev.is_playing {
                PlaybackStatus::Playing
            } else if player.has_loaded_audio() {
                PlaybackStatus::Paused
            } else {
                PlaybackStatus::Stopped
            };
            updater_integ.set_playback(last, Some(Duration::from_secs(ev.position)));
            updater_integ.set_volume(ev.volume as f64);
        }

        loop {
            match bus.recv().await {
                Ok(CoreEvent::TrackStarted { track, position_secs }) => {
                    updater_integ.set_metadata(&track_meta(&track));
                    last = PlaybackStatus::Playing;
                    updater_integ.set_playback(last, Some(Duration::from_secs(position_secs)));
                }
                Ok(CoreEvent::PlaybackStateChanged { state }) => {
                    last = map_state(state);
                    updater_integ.set_playback(last, None);
                }
                Ok(CoreEvent::PositionUpdated { position_secs, .. }) => {
                    // Keep the widget's progress bar live while playing.
                    if last == PlaybackStatus::Playing {
                        updater_integ.set_playback(last, Some(Duration::from_secs(position_secs)));
                    }
                }
                Ok(CoreEvent::VolumeChanged { volume }) => updater_integ.set_volume(volume as f64),
                Ok(_) => {}
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return,
            }
        }
    });

    Some(MprisHandle { integration, updater })
}
