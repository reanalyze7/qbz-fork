# crates/qbzd/src/tui/app.rs (1897 lines)

## Summary
The qbzd setup-TUI's top-level state machine: the `App` struct owning the
seven-section sidebar/content shell (Account/Audio/Playback/Network/Bundle/
Wizard/Scrobbler), the dual-focus (Nav/Content) key routing, the dirty-save
guard, overlays (Help/Result/DirtyLeave/ConfirmAbandon), the worker-thread
plumbing (async saves, imports/exports, HiFi Wizard probes), and the ratatui
render tree (header/breadcrumb/sidebar/content/footer/help-bar/overlays),
plus its own `#[cfg(test)]` module (~270 lines) covering pure helpers and
render smoke tests.

## Proposed split
Turn into a `tui/app/` directory:

- `tui/app/mod.rs` (~90 lines) — module doc, `pub use` re-exports of
  everything below so `crate::tui::app::App`/`Screen`/`ScreenAction`/
  `LoopCmd` keep their current paths, plus the shared vocabulary that
  doesn't fit elsewhere: `Screen` enum, `SCREENS` const, `Focus` enum.
- `tui/app/nav.rs` (~130 lines) — the pure navigation-intent layer:
  `NavIntent`, `classify_key`, `initial_focus`, `section_index`,
  `section_title`, `breadcrumb_nodes`, `sidebar_dirty_marker`,
  `cred_file_path`. These are the already-pure, already-tested helpers —
  natural first extraction.
- `tui/app/messages.rs` (~60 lines) — `ScreenAction`, `Msg`, `LoopCmd`,
  `DrawCtx`, `Active` enum, `Overlay`/`LeaveTarget` enums (the vocabulary
  the state machine and screens communicate through).
- `tui/app/state.rs` (~330 lines) — the `App` struct definition + `impl App`
  non-render methods: `new`, `refresh_status`, `derive_auth`, `enter_screen`,
  `request_section`, `leave_quit`, `enter_nav_focus`, `move_cursor`,
  `apply_leave`, `active_is_dirty`, `active_is_editing`,
  `active_editing_label`, `content_uses_horizontal`, `on_key`,
  `dispatch_screen_key`, `handle_screen_action`, `roots`,
  `refresh_scrobbler`, `after_browser_login`, `should_quit`, `busy`.
- `tui/app/save.rs` (~90 lines) — `save_active` + the worker-spawning
  actions tied to persistence: `spawn_devices`, `spawn_token_login`,
  `do_logout`, `spawn_import_plan`, `spawn_import_apply`, `spawn_export`.
- `tui/app/wizard_workers.rs` (~80 lines) — `spawn_wizard_health`,
  `spawn_wizard_detect`, `spawn_wizard_configs`, `spawn_wizard_test`
  (FB4 HiFi Wizard worker spawns — grouped since they're one cohesive
  feature slice).
- `tui/app/worker_results.rs` (~140 lines) — `drain_worker`, `on_msg` (the
  big match over `Msg` variants).
- `tui/app/draw.rs` (~220 lines) — `draw`, `draw_header`, `draw_breadcrumb`,
  `draw_sidebar`, `draw_footer`, `help_text` (all `impl App` render methods;
  Rust allows splitting `impl App` across files as long as they're in the
  same crate).
- `tui/app/worker_fns.rs` (~230 lines) — the free (non-method) async/blocking
  worker functions: `fetch_status`, `enumerate_devices`, `load_audio`,
  `write_keys`, `save_network`, `do_reload`, `plan_import`, `apply_import`,
  `export_bundle`, `build_live`, `footer_state`, `playing_extra`,
  `desktop_profile_present`, `expand_tilde`.
- `tui/app/tests.rs` (~270 lines) — the entire `#[cfg(test)] mod tests`
  block, `include!`'d or declared as `#[cfg(test)] mod tests;` from
  `mod.rs`, referencing `super::*` — since tests reach into `bare_app`
  (constructs a raw `App`), this file must `use super::state::App;` etc.,
  or simplest: keep it declared in `mod.rs` so `super::*` still resolves
  everything via the re-exports.

## Re-export surface
`tui/app/mod.rs` is the target of the existing `mod app;` (or `pub mod app;`)
declaration in `crates/qbzd/src/tui/mod.rs` (or wherever it's declared).
`pub use state::App; pub use messages::{ScreenAction, Msg, LoopCmd, DrawCtx};
pub use nav::Focus;` etc. so every external caller (`super::app::App`,
`app::Screen`, `app::LoopCmd`) keeps working unchanged.

## Coupling / watch out
- `App` is one struct with methods split across `state.rs`, `save.rs`,
  `wizard_workers.rs`, `worker_results.rs`, and `draw.rs` — this is fine in
  Rust (multiple `impl App` blocks across files) but every file needs
  `use super::messages::*; use super::nav::*;` etc. Do NOT try to move the
  struct *definition* away from its primary methods; keep `struct App` in
  `state.rs` alongside `impl App::new`.
- `tx`/`rx` (mpsc channel carrying `Msg`) is created in `App::new` and read
  in `drain_worker`/`on_msg` — both must stay able to see `Msg` from
  `messages.rs`.
- The generation/dirty-guard pattern (`leave_after_save`, `busy`) is written
  from `save.rs`'s `save_active` and read from `worker_results.rs`'s
  `on_msg` — keep both aware of the exact field names on `App`.
- Test helpers `bare_app`/`render` construct `App { .. }` with every private
  field named explicitly — if any field is renamed/moved during the split,
  update `tests.rs` in lockstep (it is the only place doing this
  struct-literal construction outside `App::new`).
- `ratatui::Frame`/`Rect` types flow through `draw.rs` only; other files
  don't need the ratatui rendering imports.

## Verify after split
- `cargo build -p qbzd` and `cargo test -p qbzd tui::app::` (the existing
  test module covers pure helpers + full-shell render smoke tests at
  80x24 and 120x30 — these are the main regression net for this file).
- Manually smoke-test the qbzd TUI (`qbzd setup` or equivalent entry point):
  section navigation, dirty-save modal, HiFi Wizard flow, Scrobbler connect
  — since worker-thread interactions (real async spawns) aren't fully
  exercised by the unit tests.
