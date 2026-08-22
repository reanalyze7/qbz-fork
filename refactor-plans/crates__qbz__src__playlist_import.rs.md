# crates/qbz/src/playlist_import.rs (494 lines)

## Summary
Rust controller behind the "Import Playlist" modal (Spotify/Apple
Music/Tidal/Deezer -> Qobuz): URL-driven provider detection, a two-step
fetch-preview-then-execute flow, a generation counter guarding stale async
runs, and every pre-formatted log/status/summary string the modal displays.

## Proposed split

- `playlist_import/mod.rs` (~35 lines) — module doc (including the
  close-mid-import / generation semantics explanation), `mod` declarations,
  `pub use` re-exports.
- `playlist_import/session.rs` (~75 lines) — `Session` struct, the
  `SESSION`/`GENERATION` statics, `current_generation`, `bump_generation`,
  `open` (the modal-reset entry point — kept here since it's the one function
  that fully resets `Session`).
- `playlist_import/fetch.rs` (~110 lines) — `on_url_edited`, `on_name_edited`,
  `begin_fetch`, `apply_preview_ok`, `apply_preview_err` — the Step A
  (preview) half of the flow.
- `playlist_import/execute.rs` (~160 lines) — `ExecuteArgs`, `begin_execute`,
  `apply_event`, `apply_execute_ok`, `apply_execute_err`, `SlintSink` +
  `ImportProgressSink` impl — the Step B (execute) half plus the streaming
  progress sink. Still a bit large; if over 130 after moving, split
  `apply_execute_ok`/`apply_execute_err` (completion handling) into
  `execute_result.rs` and keep `begin_execute`/`apply_event`/`SlintSink`
  (in-flight handling) in `execute.rs`.
- `playlist_import/format.rs` (~90 lines) — `push_log`, `clear_summary`,
  `parts_line`, `provider_display_name`, `group_thousands` — every pure
  string-formatting helper, plus their `#[cfg(test)] mod tests`
  (`group_thousands_matches_to_locale_string`).

## Re-export surface
`playlist_import/mod.rs` re-exports `open`, `on_url_edited`, `on_name_edited`,
`begin_fetch`, `apply_preview_ok`, `apply_preview_err`, `begin_execute`,
`apply_event`, `apply_execute_ok`, `apply_execute_err`, `ExecuteArgs`,
`SlintSink`, `current_generation` at `crate::playlist_import::*` so
`main.rs`'s modal-callback wiring is unaffected.

## Tricky coupling / watch out
- `SESSION` (the `Mutex<Session>`) and `GENERATION` (the `AtomicU64`) are
  shared across every file in the split (`session.rs` defines them,
  `fetch.rs`/`execute.rs` lock/read them) — keep them in `session.rs` and
  have the others reference `super::session::{SESSION, GENERATION}`
  (or re-export just the accessor functions and keep the statics module-private).
- The generation check in `SlintSink::emit` (`execute.rs`) vs. the
  `bump_generation()` calls in `open` (`session.rs`) and `begin_execute`
  (`execute.rs`) is the entire correctness mechanism behind §1.8
  (close-mid-import safety) — do not let a stale/duplicated generation
  counter get introduced by the split.
- `Session.last_logged_percent` (5%-milestone tracker) is mutated inside
  `apply_event`'s `Progress` arm (`execute.rs`) but reset in `begin_execute`
  (also `execute.rs`, good) — if `apply_event` moves to a different file than
  `begin_execute`, keep this coupling in mind.
- `push_log` (in `format.rs`) is called from `fetch.rs` and `execute.rs` —
  fine as a cross-file call once re-exported, just don't duplicate it.

## What to verify after the real split
- `cargo test -p qbz playlist_import` (the `group_thousands` test must stay
  green).
- `cargo build -p qbz` and grep for `crate::playlist_import::` in `main.rs`
  (modal open/callback wiring) to confirm nothing broke.
- Manual smoke test via the `run` skill: open the Import Playlist modal,
  paste a provider URL, preview, rename, execute, and verify the log/summary
  block and toast still render — this flow has no automated test coverage
  today so a manual pass is the real verification.
