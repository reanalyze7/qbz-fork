# crates/qbz/src/keybindings.rs (560 lines)

## 1. Summary
Rust port of the Tauri `keybindingsStore`: the 26-action table + defaults,
the shortcut-string grammar (winit key → canonical string, conflict
detection), the Slint model builder for the cheatsheet/customize-editor, the
key-capture widget handler, and the global hotkey dispatcher (action IDs →
actual app calls like play/pause, seek, nav).

## 2. Proposed module split
Following the file's own `// ==== section ====` banners (already a clean
domain cut):

| New file | Owns | ~lines |
|---|---|---|
| `keybindings/mod.rs` | Module decls + re-exports; module doc comment | ~25 |
| `keybindings/actions.rs` | `Category`, `Context` enums, `ActionDef` struct, the `ACTIONS` table, `action()` lookup | ~90 |
| `keybindings/mods.rs` | The `MODS` thread-local + `set_mods`/`mods()` (modifier tracking) | ~20 |
| `keybindings/grammar.rs` | `token_from_key`, `shortcut_from_parts`, `KEY_DISPLAY`, `format_display` (the shortcut-string grammar, pure functions) | ~100 |
| `keybindings/bindings.rs` | `active_bindings`, `action_for_shortcut`, `conflicting_action`, `set_binding`, `reset_one`, `reset_all` (persistence + conflict detection over `ui_prefs`) | ~90 |
| `keybindings/model.rs` | `build_group_vec`, `modified_count`, `refresh` (the Slint state builder) | ~60 |
| `keybindings/wire.rs` | `wire()` (startup callback wiring for `KeybindingsActions`) | ~45 |
| `keybindings/capture.rs` | `handle_capture` (the "press a key" recording widget) | ~40 |
| `keybindings/dispatch.rs` | `dispatch`, `run_action`, `focus_search`, `open_link_modal`, `seek_relative`, `handle_escape` (the global hotkey handler + its action implementations) | ~110 |

## 3. Re-export / public API surface
`keybindings/mod.rs` re-exports the current public surface so `main.rs`'s
winit event wiring and any Settings-view callers keep working:

```rust
mod actions;
mod bindings;
mod capture;
mod dispatch;
mod grammar;
mod model;
mod mods;
mod wire;

pub use actions::{ActionDef, Category, Context, ACTIONS};
pub use bindings::active_bindings;
pub use capture::handle_capture;
pub use dispatch::dispatch;
pub use grammar::{format_display, shortcut_from_parts, token_from_key};
pub use model::refresh;
pub use mods::{mods, set_mods};
pub use wire::wire;
```

## 4. Tricky coupling / shared-state to watch out for
- `active_bindings()` (in `bindings.rs`) is the single source of truth every
  other module reads: `model.rs::build_group_vec`/`modified_count`,
  `capture.rs::handle_capture`, and `dispatch.rs::dispatch` all call it —
  make sure `bindings.rs` has no dependency back on any of them (it
  currently doesn't).
- `mods()` (current modifier state) is read from BOTH `capture.rs` and
  `dispatch.rs` (to build the shortcut string at keypress time) — keep
  `mods.rs` dependency-free so both can import it without a cycle.
- `capture.rs::handle_capture` and `dispatch.rs::dispatch` share almost
  identical shortcut-building logic (`token_from_key` → `shortcut_from_parts`
  → look up in `active_bindings`) — resist the urge to fully de-duplicate
  during a pure-split refactor; that's a separate behavioral change. Just
  move each function to its target file verbatim.
- `run_action`'s match arms touch FIVE different Slint globals
  (`NowPlayingState`, `NavState`, `ShellState`, `KeyboardShortcutsState`,
  `SearchState`, `LinkResolverState`) — these imports must all follow
  `run_action` into `dispatch.rs`.
- `handle_escape` encodes a specific dismiss-priority order (link modal →
  shortcuts customize → shortcuts cheatsheet → search cortinilla → exit
  multi-select → queue) — this ordering is load-bearing UX behavior, keep it
  intact as one function in `dispatch.rs`, not scattered.
- `wire()` calls `refresh(window)` at the end (from `model.rs`) and is
  itself called once at startup from `main.rs` — confirm the cross-module
  call (`super::model::refresh` or via the re-export) still resolves.

## 5. What to verify after the real split
- `cargo build -p qbz`.
- Grep for `keybindings::` outside this file (main.rs winit event handler,
  Settings > Keyboard Shortcuts view) to confirm every import path still
  resolves.
- Smoke-test in the running app: open the keyboard-shortcuts cheatsheet
  (`?`), open the customize editor, record a new binding for one action,
  verify conflict detection fires when picking an already-used shortcut,
  reset one/all bindings, then verify the actual hotkeys still fire
  (play/pause, next/prev, seek, sidebar toggle, escape-to-close-topmost).
