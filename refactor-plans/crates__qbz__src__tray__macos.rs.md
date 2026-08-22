# crates/qbz/src/tray/macos.rs (331 lines)

## Summary
Hand-rolled macOS menu-bar tray (`NSStatusItem`) built directly on objc2 0.5
(not `tray-icon`/`muda`, to avoid an `MudaMenuItem` objc-class collision with
winit's bundled muda). Main-thread-only, `thread_local!`-held AppKit objects.

## Proposed split
- `mod.rs` (~40 lines) — `CTX`/`STATUS_ITEM`/`MENU_TARGET`/`MENU` statics
  and thread-locals, the icon byte constants, menu-tag constants.
- `menu_target.rs` (~35 lines) — the `declare_class!(QbzTrayMenuTarget)`
  block + its `impl QbzTrayMenuTarget::new`.
- `dispatch.rs` (~50 lines) — `dispatch_tag`, `handle_status_click`,
  `pop_up_menu` (routing clicked items / clicks to the shared `tray` module
  dispatch helpers).
- `icon.rs` (~40 lines) — `icon_for`, `make_image`, `apply_icon`,
  `set_icon_theme`.
- `create.rs` (~90 lines) — `create` (the main menu/status-item builder —
  the single biggest function, kept whole since it's one linear build
  sequence with real internal ordering dependencies).
- `activation.rs` (~25 lines) — `ensure_regular_active_app`,
  `set_dock_icon_hidden`.

## Re-export surface
`mod.rs` (i.e. `crate::tray::macos`, gated `#[cfg(target_os = "macos")]` at
the parent `tray/mod.rs` level) keeps exporting `create`, `set_icon_theme`,
`set_dock_icon_hidden` — the only three functions the parent `tray` module
calls.

## Coupling / watch-outs
- `CTX`, `STATUS_ITEM`, `MENU_TARGET`, `MENU` are read across nearly every
  proposed file (`dispatch.rs`, `icon.rs`, `create.rs`) — must stay declared
  in `mod.rs` with `pub(super)` visibility so siblings can reach them.
- Everything here is `!Send`/main-thread-only by construction
  (`thread_local!`, `MainThreadMarker`) — splitting into files doesn't change
  that, but a reviewer should double-check no accidental `Send`-bound
  helper gets extracted that breaks the main-thread-only invariant.
- This file can only be compiled/tested on macOS — verification of the
  split (beyond `cargo check` reading the code) requires a macOS runner;
  flag this explicitly for the implementer.

## Verify after split
`cargo check -p qbz --target x86_64-apple-darwin` (or on an actual macOS
box) after the split; manual smoke-test: launch the app on macOS, confirm
the menu-bar icon appears, left-click toggles the window, right-click pops
the menu, each menu item (Play/Pause, Next, Previous, Show/Hide, Quit)
still fires.
