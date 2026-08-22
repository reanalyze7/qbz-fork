# crates/qbzd/src/cli/search.rs (210 lines)

## Summary
The `qbzd search` CLI verb: one `GET /api/search` request rendered three ways
(human top-hits table, `--ids` newline list, `--json` raw payload), plus its
unit tests.

## Proposed split
Only modestly over budget (210 lines), and entirely because of its own
`#[cfg(test)] mod tests` block (154-210, ~57 lines). Split test-vs-production
code rather than splitting the rendering logic itself (which is cohesive and
would be artificial to break apart further).

- `cli/search.rs` (~130 lines) — keep `search()` (the async entry point,
  15-51), `CATEGORIES` (58-63), `render` (69-105), `collect_ids` (111-128),
  `secondary_name` (132-141), `id_str` (146-152). This is the file's real
  logic and sits right at/near the 130 line budget once tests are removed —
  if it's still a few lines over after removing tests, no further split is
  warranted; a single cohesive CLI-verb file at ~130-140 lines is a
  reasonable one-time overage to flag rather than fragment further.
- `cli/search/tests.rs` OR keep tests inline but trimmed: the cleanest option
  given this crate's flat `cli/` module layout (siblings are individual verb
  files, not directories) is a sibling `cli/search_tests.rs`
  declared as `#[cfg(test)] mod search_tests;` from `cli/search.rs`, OR
  (simpler, avoids adding a new module declaration) leave tests inline since
  `#[cfg(test)]` code is stripped from release builds and arguably shouldn't
  count against the 130-line production-code budget in the first place —
  flag this ambiguity for the human doing the split rather than guessing;
  the project's own convention across other files in this run (e.g.
  `qbz-external-reco/src/validate.rs`) keeps small test blocks inline, so
  precedent favors leaving tests where they are and treating `search.rs` as
  already effectively "at budget" for production code.

## Re-export surface
No change needed either way — `cli/mod.rs`'s existing `pub mod search;`
resolves to `cli/search.rs` unchanged if tests stay inline, or is unaffected
if a `search_tests` submodule is added underneath (a `mod search_tests;`
inside `search.rs` doesn't change `cli/mod.rs` at all).

## Coupling / watch out
- `render`/`collect_ids` both iterate `CATEGORIES` in the same fixed order
  (tracks first, "the composition currency for `... | qbzd queue add -`" per
  the doc comment) — this ordering is a deliberate CLI contract other scripts
  may depend on; do not reorder `CATEGORIES` while splitting.
- `id_str` handles both string (album) and numeric (track/artist/playlist)
  JSON id shapes — if extracted anywhere, keep it next to `collect_ids` and
  `render` since both call it and both need it to stay byte-identical
  (mixing string/numeric id formatting differently between the two callers
  would break the `--ids | qbzd queue add -` pipeline contract).

## Verify after split
- `cargo test -p qbzd` — the 4 existing unit tests
  (`collect_ids_leads_with_tracks_then_albums`, `id_str_handles_string_and_
  numeric_ids_without_quotes`, `render_shows_present_categories_with_ids_
  and_names`, `render_empty_payload_says_no_results`) must stay green.
- `cargo build -p qbzd` and `cargo clippy -p qbzd`.
- Smoke-test: `qbzd search "feather"` (human table), `--ids` and `--json`
  modes, against a real or mocked daemon, still produce identical output to
  before the split.
