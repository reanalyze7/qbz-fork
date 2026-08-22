# crates/qbzd/src/cli/browse.rs (220 lines)

## Summary
The `qbzd` CLI's catalog READ verbs (`album`, `artist`, `similar`, `suggest`)
— thin stateless renderers over one GET request each, sharing a generic
JSON-payload walker/renderer (human list / `--ids` / `--json` modes).

## Proposed split
By verb-commands vs. shared rendering engine — the four public commands are
already short (each ~5-15 lines); the bulk of the file is the shared
`render`/`collect_ids`/`walk` machinery plus its tests:

- `browse/mod.rs` (~60 lines) — module doc, imports, the four public command
  fns (`album`, `artist`, `similar`, `suggest`) and `to_similar_query`,
  `read_stdin_ids` (small helpers used only by `suggest`/`similar`), `mod`
  wiring (`mod render;`), re-exports.
- `browse/render.rs` (~95 lines) — `get_and_render` (the shared GET+dispatch
  helper used by all four verbs), `render`, `collect_ids`, `walk`,
  `secondary`, `id_str` (the generic items/tracks-array payload walker).
- `browse/tests.rs` (~35 lines) — the `#[cfg(test)] mod tests` block, wired
  via `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`browse/mod.rs` — becomes `crates/qbzd/src/cli/browse/mod.rs`. Keeps
`pub async fn album`, `pub async fn artist`, `pub async fn similar`,
`pub async fn suggest` directly in `mod.rs` (unchanged — these are the CLI
dispatch table's entry points, presumably called as `cli::browse::album(...)`
etc. from the arg-parsing/dispatch code) so no re-export gymnastics needed
for the four commands. `render` and `collect_ids` are currently
`pub(crate)` (used by `cli::fav` per the file's own comment: "Shared with
`cli::fav` (favorites payloads are the same items-array shape)") — after the
split these move to `browse/render.rs` and must be re-exported as `pub(crate)
use render::{render, collect_ids};` in `mod.rs` so `crate::cli::browse::
render`/`crate::cli::browse::collect_ids` (or however `cli::fav` currently
imports them) keep resolving.

## Coupling / watch out
- **This is the one file in this batch with a confirmed EXTERNAL caller**:
  the file's own doc comment says `render`/`collect_ids` are "Shared with
  `cli::fav`". Before finalizing, grep `crates/qbzd/src/cli/fav.rs` (or
  wherever `cli::fav` lives) for `browse::render` / `browse::collect_ids` /
  `use super::browse` to confirm the exact import path used today, and match
  it exactly in the re-export (`mod.rs` must re-export at the SAME path
  `cli::browse::render`/`cli::browse::collect_ids`, not
  `cli::browse::render::render`).
- `get_and_render` calls `ApiClient` (from `crate::cli::client`) and
  `ProfileRoots` (from `crate::paths`) — both imports need repeating in
  `render.rs` after the split (currently imported once at the top of
  `browse.rs`).
- `walk` is recursive and the ONLY consumer of both `render` and
  `collect_ids` — keep all three (`walk`, `render`, `collect_ids`) plus their
  small helpers (`secondary`, `id_str`) together in `render.rs`; splitting
  `walk` out alone would just add an import hop for no benefit.
- `to_similar_query` and `read_stdin_ids` are used only by the `similar`/
  `suggest` command fns respectively — fine to leave in `mod.rs` next to
  their single caller rather than moving to `render.rs` (they're
  CLI-argument-shaping, not payload-rendering, logic).

## Verify after split
- `cargo test -p qbzd cli::browse` — all 3 existing tests green
  (`to_similar_query_builds_artist_and_album_paths`,
  `walk_collects_items_and_top_level_tracks_only`, `render_empty_says_no_results`).
- `cargo check -p qbzd` — specifically confirm `cli::fav` (or whatever module
  reuses `render`/`collect_ids`) still compiles; this is the one file here
  where a naive split genuinely risks breaking a real external caller, not
  just a hypothetical one.
- Smoke-test the CLI: `qbzd album <id>`, `qbzd artist <id> --top`, `qbzd
  similar artist:<id>`, `qbzd suggest` in each of the three render modes
  (default human list, `--ids`, `--json`).
