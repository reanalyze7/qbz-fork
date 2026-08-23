use crate::state::{AuthState, LatchedErrors};

use super::device_cache::device_is_present;
use super::labels::{backend_label, bitperfect_label};
use super::{AudioStatus, AuthStatus, NetworkStatus, PlaybackStatus, StatusDoc};

/// Compose [`StatusDoc`] from live sources: `DaemonShared` (auth/latched
/// errors/tick age), the Player's sync getters via `get_playback_event`, the
/// queue via an async core call (`block_on` on the daemon runtime — this is a
/// plain serving thread, never a tokio worker, so no panic), and the audio
/// store + TTL device cache for the audio block.
pub(super) fn assemble_live(state: &crate::api::ApiState) -> StatusDoc {
    // 1. snapshot DaemonShared, then DROP the guard before any block_on so the
    //    mutex is never held across an await point.
    let (auth, user_id, subscription, last_errors, tick_age, muted, uptime, network_online) =
        match state.shared.lock() {
            Ok(s) => (
                s.auth,
                s.user_id,
                s.subscription.clone(),
                s.last_errors.clone(),
                s.driver_last_tick.map(|t| t.elapsed().as_millis() as u64),
                s.muted,
                s.started_at.elapsed().as_secs(),
                s.network_online(),
            ),
            Err(_) => (
                AuthState::Restoring,
                None,
                None,
                LatchedErrors::default(),
                None,
                false,
                0,
                true,
            ),
        };

    // 2. live player snapshot (all sync atomics, folded into one PlaybackEvent).
    let player = state.runtime.core().player();
    let ev = player.get_playback_event();
    let device_open = player.state.current_device().is_some();

    // 3. queue — async core read, driven from this non-worker thread.
    let queue = state.rt.block_on(state.runtime.core().get_queue_state());

    // 4. audio config from the store; device_present from the TTL cache. An OPEN
    //    device counts as present: CPAL enumerates DESCRIPTIONS ("HiFiBerry DAC+
    //    ..."), never `hw:CARD=...` ids, so a playing direct-hw stream would
    //    otherwise report `not present` (false negative).
    let settings = state.audio.get_settings().ok();
    let backend = settings.as_ref().and_then(|s| backend_label(s.backend_type));
    let configured_device = settings.as_ref().and_then(|s| s.output_device.clone());
    let device_present = match &configured_device {
        None => true, // system default is always "present"
        Some(dev) => device_open || device_is_present(state, dev),
    };

    // 5. playback block. `stopped` when nothing is loaded and the queue has no
    //    current track; otherwise `playing`/`paused`.
    let has_track = queue.current_track.is_some();
    let pstate = if ev.is_playing {
        "playing"
    } else if has_track || player.has_loaded_audio() {
        "paused"
    } else {
        "stopped"
    };
    let stopped = pstate == "stopped";
    let (title, artist, track_id) = match &queue.current_track {
        Some(t) => (Some(t.title.clone()), Some(t.artist.clone()), Some(t.id)),
        None if ev.track_id != 0 => (None, None, Some(ev.track_id)),
        None => (None, None, None),
    };

    StatusDoc {
        version: env!("CARGO_PKG_VERSION").to_string(),
        api_version: crate::API_VERSION,
        uptime_secs: uptime,
        data_root: state.roots.data.display().to_string(),
        driver_tick_age_ms: tick_age,
        auth: AuthStatus {
            state: auth,
            user_id,
            subscription,
        },
        audio: AudioStatus {
            backend,
            configured_device,
            device_present,
            device_open,
            bit_perfect: bitperfect_label(ev.bit_perfect_mode),
            sample_rate: ev.sample_rate,
            bit_depth: ev.bit_depth,
        },
        playback: PlaybackStatus {
            state: pstate.to_string(),
            track_id: if stopped { None } else { track_id },
            title: if stopped { None } else { title },
            artist: if stopped { None } else { artist },
            position: if stopped { None } else { Some(ev.position) },
            duration: if stopped { None } else { Some(ev.duration) },
            volume: ev.volume,
            muted,
            queue_len: queue.total_tracks,
        },
        network: NetworkStatus { online: network_online },
        last_errors,
    }
}
