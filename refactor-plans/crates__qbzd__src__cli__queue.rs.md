# crates/qbzd/src/cli/queue.rs (509 lines)

## Summary
The `qbzd queue {list,add,remove,clear,move,jump,stop-after}` CLI verbs — each verb
is one HTTP call to the daemon's `/api/queue/*` endpoints, plus pure
rendering/index-translation helpers (`cli_index_to_api`, `cli_position`,
`render_queue_list`, etc.) that are unit-tested without a running daemon.

## Proposed split
The file is already banner-divided by verb (`// === list ===`, `// === add ===`, ...)
plus a big `#[cfg(test)]` block (~170 lines). Split by verb-group + pull tests out,
following the pure(rendering)/IO(HTTP calls) split the file's own header comment
already calls out.

- `queue/mod.rs` (~20 lines) — module doc (the INDEX CONVENTION note is critical,
  keep verbatim), `use` block, re-export of all verb functions
  (`list, add, remove, clear, move_, jump, stop_after`) and the two shared
  translation helpers.
- `queue/list.rs` (~90 lines) — `list()`, `HEADER`, `HISTORY_RENDER_CAP`,
  `render_queue_list()`, `render_row()` — the read-path verb + its renderer.
- `queue/mutate.rs` (~150 lines) — `add()`, `render_added()`, `remove()`,
  `cli_index_to_api()`, `cli_position()`, `render_removed()`, `clear()` — the
  mutating verbs plus the index-translation functions they share (both `list.rs`
  and `mutate.rs`/`nav.rs` need `cli_position`/`cli_index_to_api`, so keep those two
  functions in `mod.rs` instead — see below).
- `queue/nav.rs` (~90 lines) — `move_()`, `jump()`, `stop_after()` (position-based
  navigation verbs).
- `queue/fmt.rs` (~10 lines) — `fmt_mmss()` shared formatting helper.
- `queue/tests.rs` (~170 lines) — the entire `#[cfg(test)] mod tests` block.

Revise: put `cli_index_to_api` and `cli_position` in `queue/mod.rs` itself (not
`mutate.rs`) since `list.rs` (via `render_queue_list`'s use of `cli_position`),
`mutate.rs` (`remove`), and `nav.rs` (`move_`, `jump`) all call one or both —
centralizing them in `mod.rs` avoids a cross-submodule dependency edge.

## Re-export surface
`queue/mod.rs` is the public surface: `pub use list::list; pub use mutate::{add,
remove, clear}; pub use nav::{move_, jump, stop_after};` plus keeps
`cli_index_to_api`/`cli_position` defined directly in `mod.rs` and `pub(crate)` or
`pub` as today. The `qbzd` CLI dispatch table (wherever `cli::queue::list`/`add`/etc.
are called from, likely `cli/mod.rs`) keeps using `crate::cli::queue::{list, add,
...}` unchanged.

## Coupling / watch out
- `cli_index_to_api` / `cli_position` are exact inverses (there's even a round-trip
  test) and used across at least 3 verbs (`remove`, `move_`, `jump`, plus `list`'s
  renderer) — this is the file's central shared "pure" surface; get it right in
  `mod.rs` first before splitting the verb files.
- `render_added` and `render_removed` are each used by exactly one verb — safe to
  colocate with their verb function rather than centralizing in a shared render.rs.
- All verbs take `(host: Option<String>, roots: &ProfileRoots, ...)` and construct
  `ApiClient::new(host, roots)` identically — no shared mutable state, just a
  repeated construction pattern; no special handling needed.
- The extensive doc comments citing spec sections (`§2.2`, `§3.3.13`, etc.) must
  travel with their function when split — don't leave stale references in `mod.rs`.

## Verify after split
- `cargo test -p qbzd cli::queue` — all ~11 existing tests (index translation,
  round-trip, 4 `render_queue_list` variants, `render_added` x2, `render_removed`,
  `fmt_mmss`) must stay green with identical byte-exact assertions.
- `cargo check -p qbzd` and confirm the CLI arg-parsing dispatcher (clap subcommand
  match) still resolves `queue::list/add/remove/clear/move_/jump/stop_after`.
