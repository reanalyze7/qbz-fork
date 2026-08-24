//! Shuffle + repeat-mode transport controls, split out of `transport.rs` to
//! keep both files under the line budget.
use slint::ComponentHandle;

use super::state::refresh_sidebar;
use super::Runtime;
use crate::{AppWindow, NowPlayingState};
use qbz_models::RepeatMode;

/// Toggle shuffle on the queue and reflect the new state on NowPlayingState.
pub fn toggle_shuffle(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let on = runtime.core().toggle_shuffle().await;
        let _ = weak.upgrade_in_event_loop(move |w| {
            w.global::<NowPlayingState>().set_shuffle(on);
        });
        // The ONLY visible effect of a shuffle toggle is the reordered UP NEXT
        // list (the current track stays current, so the NPB itself doesn't
        // change). That list lives in QueueState.upcoming-page / coverflow-tracks
        // and is pushed ONLY by the queue controller, so without this re-pull the
        // button looked dead. `false` skips the network favorite re-pull (shuffle
        // never changes favorites) — cheap and offline-safe.
        refresh_sidebar(false);
    });
}

/// Cycle the repeat mode Off -> All -> One -> Off and reflect it.
pub fn cycle_repeat(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let next = match runtime.core().get_queue_state().await.repeat {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
        runtime.core().set_repeat_mode(next).await;
        let mode: i32 = match next {
            RepeatMode::Off => 0,
            RepeatMode::All => 1,
            RepeatMode::One => 2,
        };
        let _ = weak.upgrade_in_event_loop(move |w| {
            w.global::<NowPlayingState>().set_repeat_mode(mode);
        });
    });
}
