# crates/qbzd/src/tui/screens/network.rs (322 lines)

## Summary
The TUI "Network" settings screen: edits `[server]` bind/port/token in
`qbzd.toml` via a whole-file parse-update-rewrite that preserves every other
key (including unrecognized ones), with a LAN-exposure warning and
field-level validation.

## Proposed split

- `network/mod.rs` (~20 lines) — module doc, `mod` declarations, `pub use`
  re-exports of `NetworkState` and `rewrite_toml`.
- `network/state.rs` (~120 lines) — `Staged`, `NField`, `FIELDS`,
  `NetworkState` struct + its non-render, non-input methods: `new`,
  `is_dirty`, `is_editing`, `mark_saved`, `editing_label`, `validated`,
  `bind_is_lan`, `port_invalid`.
- `network/input.rs` (~45 lines) — `NetworkState::handle_key` (moved into a
  second `impl NetworkState` block in this file — Rust allows multiple
  `impl` blocks for the same type split across files in one crate).
- `network/draw.rs` (~100 lines) — `NetworkState::draw` and its private
  helper `field_value` (another `impl NetworkState` block).
- `network/toml_rewrite.rs` (~35 lines) — the free function `rewrite_toml`.
- `network/tests.rs` (~45 lines) — the `#[cfg(test)] mod tests` block
  (`rewrite_preserves_unknown_and_known_keys`, `empty_token_clears_the_key`,
  `bad_ip_and_port_are_rejected`).

## Re-export surface
`network/mod.rs` re-exports `NetworkState` and `rewrite_toml` at
`crate::tui::screens::network::*` — the TUI app shell (`tui/app.rs`) that
constructs `NetworkState::new(...)` and calls `handle_key`/`draw`/`is_dirty`/
`mark_saved`/`editing_label` needs no changes.

## Tricky coupling / watch out
- `NetworkState` is split across three `impl` blocks (`state.rs`, `input.rs`,
  `draw.rs`) — all three must stay in the same crate/module tree (they will,
  as submodules of `network/`) since private-field access requires that.
  Rust does NOT require `impl` blocks to be in the same *file*, only visible
  to code that needs the fields — since these are all within
  `network/` and `NetworkState`'s fields are private to the module, this
  works as long as none of the split files live outside `crates/qbzd/src/tui/
  screens/network/`.
- `draw`'s LAN-exposure and validation-note rendering
  (`self.staged.bind.parse::<IpAddr>()`, `self.bind_is_lan()`,
  `self.port_invalid()`) calls back into `state.rs`'s methods — keep these
  method signatures unchanged (`&self` only, no interior mutability) so the
  cross-file calls compile without friction.
- `handle_key`'s field-editor overwrite (`self.staged.token = input.buf.clone()`)
  bypasses `TextInput`'s normal trim — this is deliberate (token values may
  need leading/trailing whitespace preserved unlike bind/port) — don't
  "consolidate" the three match arms into one during the split.
- The pre-save unknown-key warning (`unknown_keys`) is threaded from
  `NetworkState::new` (constructor, in `state.rs`) through to `draw` (in
  `draw.rs`) — a plain field read, no special handling needed but worth
  flagging since it's the one piece of state that comes from OUTSIDE the
  config file being edited (the caller's pre-parse of unknown TOML keys).

## What to verify after the real split
- `cargo test -p qbzd tui::screens::network` — all three tests green,
  especially `rewrite_preserves_unknown_and_known_keys` (the core J5
  silent-revert guard).
- `cargo build -p qbzd` and grep for `screens::network::` /
  `NetworkState` in `crates/qbzd/src/tui/app.rs` (or wherever the screen
  router lives) to confirm the public path is unchanged.
- Manual smoke test via the `run` skill if the TUI can be driven
  non-interactively, or at minimum a manual terminal check: open the Network
  screen, edit bind to a non-loopback address (confirm the LAN warning
  appears), enter an invalid port (confirm the error line appears), save,
  and confirm `qbzd.toml`'s other sections/keys are untouched.
