# crates/qbz/src/tray/mod.rs (354 lines)

## Summary
System tray orchestration (cross-platform dispatch over `linux`/`macos`
submodules): `TrayHandle` (live tray control), process-global init/lifecycle,
window show/hide/toggle/present with Wayland-specific workarounds, and
player-action dispatch (play/pause/next/previous/volume) forwarding to the
playback controller.

## Proposed split
By responsibility — this file already groups itself with `// --- section ---`
banner comments, which map directly to module seams.

- `tray/mod.rs` (~50 lines) — module doc, `mod linux`/`mod macos` platform
  gates, `pub(crate) type Runtime`, statics (`WINDOW_SHOWN`, `LAST_TOGGLE`,
  `TOGGLE_DEBOUNCE_MS`, `TRAY` OnceLock), `pub fn handle()`, and `pub use`
  re-exports from the split files so `crate::tray::init`,
  `crate::tray::toggle_window`, etc. keep working.
- `tray/handle.rs` (~55 lines) — `TrayHandle` struct + its impl
  (`set_track`/`clear_track`/`set_playing`/`set_icon_theme`), the
  cross-platform live-update handle.
- `tray/init.rs` (~65 lines) — `pub fn init(...)`: the platform-gated setup
  (Linux std::thread + ksni, macOS event-loop create, no-op fallback) and the
  `INIT_STARTED` guard.
- `tray/window.rs` (~110 lines) — the "Window show/hide" section:
  `toggle_window`, `show_window`, `present`, `hide_window`,
  `set_window_shown`, `set_mac_dock_hidden`, `quit`. All the winit/Slint
  window-visibility + debounce logic.
- `tray/dispatch.rs` (~55 lines) — the "Player-action dispatch" section:
  `dispatch_play_pause`, `dispatch_next`, `dispatch_previous`,
  `dispatch_volume_delta` (Linux-only).

## Re-export surface
`tray/mod.rs` stays the public surface: `pub use handle::TrayHandle; pub use
init::init; pub use window::{toggle_window, show_window, present, hide_window,
set_window_shown, set_mac_dock_hidden, quit}; pub use
dispatch::{dispatch_play_pause, dispatch_next, dispatch_previous,
dispatch_volume_delta};` (last one behind the same `#[cfg(target_os =
"linux")]`). Every external call site (`crate::tray::init(...)`,
`crate::tray::toggle_window(...)`, MPRIS's use of `crate::tray::present`, the
window close-handler's `crate::tray::set_window_shown`) needs zero changes.

## Coupling / watch out
- `WINDOW_SHOWN` (tray/mod.rs) is read/written by BOTH `window.rs`
  (`toggle_window`/`show_window`/`hide_window`/`set_window_shown`) — keep it
  defined once in `mod.rs` as `pub(super)` so `window.rs` can reach it, or move
  it INTO `window.rs` directly (cleaner — it's only ever touched there) and
  have `mod.rs` not need it at all.
- `LAST_TOGGLE`/`TOGGLE_DEBOUNCE_MS` are only used in `window.rs::toggle_window`
  — safe to move both into `window.rs` entirely rather than keeping in `mod.rs`.
- `TRAY` OnceLock is set from `init.rs` (on successful platform init) and read
  from `mod.rs::handle()` — must stay visible across the two files
  (`pub(super)` static in `mod.rs`, or move to `init.rs` and re-export the
  getter).
- `dispatch_volume_delta` is `#[cfg(target_os = "linux")]`-gated because
  scroll-to-volume is a StatusNotifierItem-only feature — preserve the cfg gate
  exactly when moving to `dispatch.rs`.
- `hide_window` clears `ImmersiveState.shader_texture` with a detailed comment
  about cross-instance wgpu texture reuse (a real crash-avoidance fix, not
  incidental) — keep that comment attached verbatim when moving to `window.rs`.
- The Linux `init` path spawns a dedicated `std::thread` specifically to avoid
  calling ksni's blocking `spawn()` (which internally does `Runtime::block_on`)
  from inside an existing tokio context — keep this explanation attached to
  `init.rs`'s Linux branch.

## Verify after split
- `cargo build -p qbz` on Linux (the primary target given the ksni Linux
  submodule) and ideally macOS if available.
- Smoke-test: `grep -rn "tray::" crates/qbz/src` still resolves — check
  `tray::init`, `tray::present` (MPRIS), `tray::handle()`,
  `tray::set_window_shown`, `tray::toggle_window`.
- Manual smoke-test: tray icon appears, left-click toggles the window (with the
  double-click debounce not firing twice), tray menu play/pause/next/previous
  drive the SAME playback as the in-app player bar, Linux scroll-to-volume on
  the tray icon works, window hide/show doesn't crash the dynamic background
  shader (the wgpu texture cross-instance bug the comment warns about).
