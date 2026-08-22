# crates/qbzd/src/tui/screens/playback.rs (553 lines)

## Summary
The `qbzd` setup-TUI Playback screen: staged-settings model
(`StagedPlayback`, `PField`, visibility/enable rules), key-input handling
(focus nav + inline select popups for Quality/MaxRate/RetryFail), rendering
(grouped sections), label mappers, and tests.

## Proposed split
By responsibility: model/state, input handling, rendering, label mappers,
tests — matching the file's own `// -------- input --------` / `// --------
render --------` section markers.

- `screens/playback/mod.rs` (~40 lines) — module doc, `pub mod`
  declarations, `pub use` re-exports of `StagedPlayback`, `PField`,
  `row_state`, `visible_fields`, `PlaybackState` so
  `crate::tui::screens::playback::X` paths are unchanged.
- `screens/playback/model.rs` (~90 lines) — `MAX_RATES`, `StagedPlayback`,
  `PField`, `row_state`, `visible_fields`, `Editor` enum,
  `PlaybackState` struct definition + `new`/`is_dirty`/`is_editing`/
  `mark_saved`/`editing_label`.
- `screens/playback/save.rs` (~50 lines) — `PlaybackState::save_keys` (the
  dirty-diff-to-dotted-keys method) as its own `impl PlaybackState` block.
- `screens/playback/input.rs` (~135 lines) — `handle_key`, `activate`,
  `handle_editor_key` as an `impl PlaybackState` block.
- `screens/playback/render.rs` (~130 lines) — `draw`, `group_block`,
  `field_block`, `field_display`, `field_description` as an `impl
  PlaybackState` block (+ the standalone `field_description` fn).
- `screens/playback/labels.rs` (~45 lines) — `quality_label`,
  `max_rate_label`, `retry_label`, `autoplay_label`, `autoplay_value`.
- `screens/playback/tests.rs` (~65 lines) — the `#[cfg(test)] mod tests`
  block (`base` helper + the 5 tests).

## Re-export surface
`screens/playback/mod.rs` re-exports `StagedPlayback`, `PField`,
`row_state`, `visible_fields`, `PlaybackState` at
`crate::tui::screens::playback::*` — the TUI's screen-router (`tui/app.rs`
or similar, matching on the current screen) constructs `PlaybackState` and
calls `.handle_key`/`.draw`/`.save_keys`/`.is_dirty`/`.mark_saved` on it;
that call site is unaffected as long as the type and its inherent methods
stay reachable at the same path.

## Coupling / watch out
- `PlaybackState` is one struct with its `impl` split across
  `model.rs`/`save.rs`/`input.rs`/`render.rs` — Rust allows multiple
  `impl` blocks for the same type across files in one crate, but every
  field (`baseline`, `staged`, `focus`, `editor`) must stay `pub(crate)`
  or otherwise visible to all four impl-block files (they're currently
  private fields accessed only within the same module — once split into
  sibling files under `screens/playback/`, private fields ARE still
  visible to sibling modules in Rust as long as they're all descendants
  of the same parent module, so no visibility changes needed as long as
  everything stays under `screens/playback/`).
- `row_state`'s three special-cased fields (`MaxRate` gated on
  `limit_to_device`, `Gapless` gated on `!streaming_only`, `Resume` gated
  on `restore_session`) encode the §3.3 spec rules referenced in the file
  header — keep `row_state` and its three call sites (`visible_fields`,
  `activate`, `field_block`) logically together; don't let `render.rs` and
  `model.rs` drift on which gates which.
- `save_keys`'s `fallback_behavior != "ask"` guard is the one place the
  "TUI never writes ask" spec rule (§3.3.2, called out three times in
  comments across the file) is enforced — keep this exact conditional
  when moving to `save.rs`, and keep the `ask_renders_note_and_is_never_
  written` test alongside it conceptually even though it physically lives
  in `tests.rs`.
- `Editor` enum (in `model.rs`) is matched exhaustively in both
  `input.rs` (`handle_editor_key`) and `render.rs` (`draw`'s editor-popup
  match) — adding a new editor variant later means touching three files;
  worth a one-line note in `model.rs` pointing at the other two.

## Verify after split
- `cargo test -p qbzd tui::screens::playback::` — all 5 tests green.
- `cargo check -p qbzd` to confirm the screen-router's construction/call
  sites into `PlaybackState` still compile.
- Manual smoke-test: run the setup TUI, navigate to the Playback screen,
  toggle "limit to device" (confirm Max Rate row appears/hides), open the
  Quality/MaxRate/RetryFail select popups and pick a value, toggle
  Gapless while Streaming-only is on (confirm it's disabled with the
  reason text), save and confirm the daemon settings actually change.
