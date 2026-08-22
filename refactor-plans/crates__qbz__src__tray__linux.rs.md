# crates/qbz/src/tray/linux.rs (485 lines)

## Summary
Linux system tray via `ksni` (StatusNotifierItem): embeds/decodes themed tray
icon pixmaps (mono-black/mono-white/color x 4 sizes), detects system dark-mode,
implements the `ksni::Tray` trait (icon, tooltip, click/scroll/menu handlers),
and runs a dedicated updater thread that applies live tooltip/icon-theme updates
via an `mpsc` channel (to dodge a `ksni::blocking` + tokio re-entrancy panic).

## Proposed split
By responsibility — embedded assets/icon decoding ↔ dark-mode detection ↔ the
`Tray` trait impl ↔ the cross-thread update channel:

- `tray/linux/mod.rs` (~20 lines) — module doc, `mod` declarations, `pub use
  init, LinuxTrayHandle`.
- `tray/linux/icons.rs` (~140 lines) — all 12 `include_bytes!` constants,
  `decode_pixmap`, `IconVariant` enum, `resolve_variant`, `decode_tray_icons`
  (everything about turning embedded PNGs into `ksni::Icon` pixmaps).
- `tray/linux/dark_mode.rs` (~45 lines) — `is_flatpak`, `prefer_dark_tray` (the
  GNOME/GTK/KDE dark-scheme sniffing, self-contained and easily testable in
  isolation later).
- `tray/linux/tray_impl.rs` (~150 lines) — `NowPlaying` struct, `QbzTray` struct,
  `impl QbzTray` (`play_pause`), `impl Tray for QbzTray` (id/title/icon_name/
  icon_pixmap/tool_tip/activate/secondary_activate/scroll/menu) — the actual
  `ksni::Tray` trait surface.
- `tray/linux/updater.rs` (~150 lines) — `TrayUpdate` enum, `LinuxTrayHandle`
  struct + its `impl` (`empty`, `install`, `send`, `set_track`, `clear_track`,
  `set_playing`, `set_icon_theme`) — the mpsc-channel + dedicated-thread pattern
  that works around the `ksni::blocking` + tokio panic.
- `tray/linux/init.rs` (~40 lines) — the `pub fn init(...)` entry point that
  wires `decode_tray_icons` + `QbzTray` + `LinuxTrayHandle::install` +
  Flatpak's `disable_dbus_name`.

## Re-export surface
`tray/linux/mod.rs` re-exports `init` and `LinuxTrayHandle` — the two symbols
`crate::tray` (the parent module, which does `#[cfg(target_os = "linux")] mod
linux;`) actually uses, per the file's own doc comment referencing
`super::dispatch_*` / `super::toggle_window` / `super::quit` (i.e. this module is
consumed by its parent, not vice versa) — so `tray/mod.rs`'s existing
`linux::init(...)` / `linux::LinuxTrayHandle` call sites need no changes.

## Coupling / watch out
- `QbzTray` (in `tray_impl.rs`) directly references `LinuxTrayHandle`'s inner
  thread only indirectly (it doesn't hold one) — but `updater.rs`'s
  `install()` takes a `ksni::blocking::Handle<QbzTray>`, so `updater.rs` must
  `use super::tray_impl::QbzTray;`. Order the split so `tray_impl.rs` has no
  dependency back on `updater.rs` (it doesn't currently).
- `decode_tray_icons` (icons.rs) is called both from `init.rs` (initial load)
  and from `updater.rs`'s `TrayUpdate::SetIconTheme` handler (live theme
  switch) — export it as `pub(super)` from `icons.rs` so both call sites work.
- `is_flatpak()` (dark_mode.rs, despite the name mismatch — it's really a
  "sandbox detection" helper, not dark-mode) is used only by `init.rs` — keep it
  `pub(super)`; consider it might fit better in its own tiny `sandbox.rs` but
  co-locating with `dark_mode.rs` is fine given both are small environment
  probes.
- The comment above `TrayUpdate` explains WHY the mpsc+thread pattern exists
  (ksni::blocking + tokio re-entrancy panic) — preserve that comment verbatim in
  `updater.rs`, it documents non-obvious behavior future readers need.
- `super::Runtime` / `crate::AppWindow` imports are used throughout — carry the
  `use super::Runtime;` / `use crate::AppWindow;` lines into whichever files
  reference them (`tray_impl.rs`, `init.rs`).

## Verify after split
- `cargo check -p qbz --target x86_64-unknown-linux-gnu` (or the default Linux
  target) since this file is Linux-only (`tray/linux.rs`) — confirm it still
  only compiles under the same `#[cfg(target_os = "linux")]` gate as before (no
  new gate needed if the parent `mod linux;` declaration already carries it).
- `cargo build -p qbz`.
- Smoke-test on Linux: launch the app, confirm the tray icon appears with the
  correct theme, tooltip shows now-playing info, left-click toggles the window,
  middle-click/scroll drive play-pause/volume, and the menu items work.
