# crates/qbzd/src/cli/play.rs (140 lines)

## Summary
The `qbzd play [CONTENT]` CLI verb: bare `play` resumes the current queue via
the shared `transport::play`; with a content argument it POSTs `/api/play`
after mapping the content token (URL, `kind:ID`, or bare numeric track id) to
a JSON body, then renders the response.

## Proposed split
Only ~10 lines over budget — the whole file is already a single cohesive
verb handler with a small "internals" section and its own tests; a light
split into the public entry point vs. the pure mapping/rendering helpers is
enough.

- `play.rs` (~45 lines, stays at this path) — module doc, the public
  `pub async fn play(...)` entry point (lines 14-43) — the only thing
  `cli/mod.rs`'s command dispatch calls. `pub use play_format::{to_body,
  render, parse_u64};` (or just keep them declared via `mod play_format;`
  with `use play_format::*;` internally) so `play()`'s body is unchanged.
- `play_format.rs` (~55 lines, new sibling file, declared via `mod
  play_format;` in `play.rs`) — the pure helpers: `to_body`, `parse_u64`,
  `render` (lines 50-101) — content-token parsing and response rendering,
  zero I/O, easy to unit test in isolation.
- `play_format.rs` also carries its own `#[cfg(test)] mod tests` (the
  existing 4 tests at lines 103-140 all exercise `to_body`/`render`, so they
  move wholesale into this file rather than staying in `play.rs`).

## Re-export surface
`play.rs` stays the single import path `cli/mod.rs`'s command-dispatch table
uses (`crate::cli::play::play(host, content, roots)`), unchanged. The
`play_format` module is an implementation detail of `play.rs`; nothing
outside this file calls `to_body`/`parse_u64`/`render` directly today (they
are not `pub(crate)` beyond this file), so no other crate/module path
changes.

## Coupling / watch out
- `play()` calls `to_body(&content)` and `render(&v)` from `play_format.rs`
  — both need at least `pub(super)` visibility (currently plain private
  `fn`) once split into a sibling file.
- `play()`'s error path prints a specific "try: qbzd play album:<ID> | …"
  hint tied directly to the same prefixes `to_body` parses — if `to_body`'s
  accepted prefixes ever change, remember to update this hint string in
  `play.rs` too (it's not derived from `play_format.rs`, just documentation
  that must be kept in sync manually).
- `parse_u64` is a tiny 3-line helper only used by `to_body` — keep both in
  `play_format.rs` together, no need for a third file.

## Verify after split
- `cargo test -p qbzd cli::play::` — all 4 existing tests
  (`to_body_maps_prefixes_and_urls`, `to_body_bare_number_is_a_track_id`,
  `to_body_rejects_non_numeric_prefixed_ids_and_garbage`,
  `render_reads_queued_and_track`) green.
- `cargo check -p qbzd` and grep for `cli::play::play` in the command
  dispatch table to confirm the public entry point path is unchanged.
- Smoke-test: `qbzd play` (resume), `qbzd play track:<id>`,
  `qbzd play album:<id>`, and a malformed selector (expect exit code 2 with
  the usage hint) against a running daemon.
