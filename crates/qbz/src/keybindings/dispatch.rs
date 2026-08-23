//! Dispatch (the global hotkey handler for the MAIN window).

use i_slint_backend_winit::{winit::keyboard::Key, EventResult};
use slint::ComponentHandle;

use crate::{
    AppWindow, KeyboardShortcutsState, LinkResolverState, NavState, NowPlayingState, SearchState,
    ShellState,
};

use super::bindings::{action_for_shortcut, active_bindings};
use super::grammar::{shortcut_from_parts, token_from_key};
use super::model::refresh;
use super::mods::mods;

/// Resolve + run a hotkey for the main window. Returns `PreventDefault` when an
/// action fired, `Propagate` otherwise. The caller has already ruled out
/// recording mode, an open search dropdown, and text-input focus.
pub fn dispatch(window: &AppWindow, key: &Key) -> EventResult {
    let (ctrl, alt, shift) = mods();
    let Some(token) = token_from_key(key) else {
        return EventResult::Propagate;
    };
    let Some(shortcut) = shortcut_from_parts(ctrl, alt, shift, &token) else {
        return EventResult::Propagate;
    };
    let bindings = active_bindings();
    let Some(action) = action_for_shortcut(&shortcut, &bindings) else {
        return EventResult::Propagate;
    };
    run_action(window, action.id);
    EventResult::PreventDefault
}

fn run_action(window: &AppWindow, id: &str) {
    match id {
        "playback.toggle" => window.global::<NowPlayingState>().invoke_toggle_play(),
        "playback.next" => window.global::<NowPlayingState>().invoke_next(),
        "playback.prev" => window.global::<NowPlayingState>().invoke_previous(),
        "nav.back" => window.global::<NavState>().invoke_request_back(),
        "nav.forward" => window.global::<NavState>().invoke_request_forward(),
        "nav.search" => focus_search(window),
        "nav.settings" => window.global::<NavState>().invoke_request_settings(),
        "ui.sidebar" => window.global::<ShellState>().invoke_cycle_sidebar(),
        "ui.queue" => {
            let shell = window.global::<ShellState>();
            let open = shell.get_queue_open();
            shell.set_queue_open(!open);
        }
        "ui.escape" => handle_escape(window),
        "ui.openLink" => open_link_modal(window),
        "ui.showShortcuts" => {
            window.global::<KeyboardShortcutsState>().set_open(true);
            refresh(window);
        }
        "focus.seekForward" => seek_relative(window, 5),
        "focus.seekBack" => seek_relative(window, -5),
        "focus.seekForwardLong" => seek_relative(window, 10),
        "focus.seekBackLong" => seek_relative(window, -10),
        _ => {}
    }
}

fn focus_search(window: &AppWindow) {
    // Open the header cortinilla; the field grabs focus on open.
    window.global::<SearchState>().set_cortinilla_open(true);
}

fn open_link_modal(window: &AppWindow) {
    let s = window.global::<LinkResolverState>();
    s.set_url("".into());
    s.set_platform("".into());
    s.set_error("".into());
    s.set_playlist_detected(false);
    s.set_playlist_provider("".into());
    s.set_resolving(false);
    s.set_open(true);
}

/// Seek by `delta` seconds (clamped).
fn seek_relative(window: &AppWindow, delta: i32) {
    let np = window.global::<NowPlayingState>();
    let duration = np.get_duration_secs();
    if duration <= 0 {
        return;
    }
    let pos = np.get_position_secs();
    let target = (pos + delta).clamp(0, duration);
    np.invoke_seek(target as f32 / duration as f32);
}

/// Close the topmost dismissable surface. Text-input focus has already been
/// ruled out by the caller, so this only touches non-text overlays.
fn handle_escape(window: &AppWindow) {
    if window.global::<LinkResolverState>().get_open() {
        window.global::<LinkResolverState>().set_open(false);
        return;
    }
    if window.global::<KeyboardShortcutsState>().get_customize_open() {
        window.global::<KeyboardShortcutsState>().set_customize_open(false);
        return;
    }
    if window.global::<KeyboardShortcutsState>().get_open() {
        window.global::<KeyboardShortcutsState>().set_open(false);
        return;
    }
    if window.global::<SearchState>().get_cortinilla_open() {
        window.global::<SearchState>().set_cortinilla_open(false);
        return;
    }
    // Leaving a multi-select session (clear + mode off) takes priority over
    // closing the queue. No-op when no surface is in select mode.
    if crate::exit_active_multi_select(window) {
        return;
    }
    let shell = window.global::<ShellState>();
    if shell.get_queue_open() {
        shell.set_queue_open(false);
    }
}
