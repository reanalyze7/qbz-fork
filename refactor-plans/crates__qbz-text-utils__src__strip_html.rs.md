# crates/qbz-text-utils/src/strip_html.rs (468 lines)

## Summary
Converts Qobuz HTML-ish prose (biographies, album reviews) into
Slint-friendly plain text: normalizes `<br>`/`</p>` into newlines, strips all
other tags, decodes HTML entities (named + numeric, including a
Windows-1252 quirk-range and a small malformed-no-semicolon allowlist), and
collapses excess blank lines — plus a large `#[cfg(test)]` module (~125
lines) and a large `NAMED` entity table (~100 lines) embedded inline.

## Proposed split
Turn into a `strip_html/` directory:

- `strip_html/mod.rs` (~30 lines) — module doc, the public `strip_html`
  and `decode_html_entities` entry points (the pipeline: normalize → strip
  → decode → collapse), `pub use` / `pub(crate) use` of internals needed
  across files, and `#[cfg(test)] mod tests;` declaration.
- `strip_html/breaks.rs` (~60 lines) — `normalize_breaks`,
  `match_break_or_paragraph` (the `<br>`/`</p>` → newline pass).
- `strip_html/tags.rs` (~15 lines) — `strip_remaining_tags` (drop all other
  tags, char-safe).
- `strip_html/entities.rs` (~110 lines) — `decode_entities`, `match_entity`,
  `match_numeric`, `BARE_NAMES` const, and the Windows-1252 quirk-range
  match block inside `match_numeric` (the entity-decoding logic, separated
  from its data table).
- `strip_html/entity_table.rs` (~100 lines) — the `NAMED` const table alone
  (core escapes, symbols, dashes/quotes, full Latin-1 accented set) — pure
  data, isolating it keeps `entities.rs` under budget and makes future
  additions to the table a single-file diff.
- `strip_html/collapse.rs` (~20 lines) — `collapse_blank_lines`.
- `strip_html/tests.rs` (~125 lines) — the entire `#[cfg(test)] mod tests`
  block verbatim (references `super::*`, which resolves via `mod.rs`'s
  re-exports).

## Re-export surface
`strip_html/mod.rs` is the target of the existing `mod strip_html;` (or
`pub mod strip_html;`) in `crates/qbz-text-utils/src/lib.rs`. It must
`pub use` (or keep as `pub fn` directly in `mod.rs`) exactly `strip_html`
and `decode_html_entities` — the only two functions used outside this
module (confirmed as the two documented public entry points; every other
function here is private/internal). No caller-visible path changes.

## Coupling / watch out
- `NAMED` and `BARE_NAMES` are both looked up by name inside
  `match_entity` in `entities.rs` — if `NAMED` moves to `entity_table.rs`,
  `entities.rs` needs `use super::entity_table::NAMED;`.
- The pipeline order in `strip_html` (normalize_breaks → strip_remaining_tags
  → decode_entities → collapse_blank_lines) is easy to get wrong if the
  functions are re-ordered/renamed during the split — keep the exact
  function names and call order in `mod.rs`.
- `decode_html_entities` (public) calls the same private `decode_entities`
  that `strip_html`'s pipeline calls — both must resolve to the one
  function in `entities.rs`, don't accidentally duplicate it.
- No cross-cutting global state (this file has zero `static`/mutable
  state) — this is a low-risk, purely functional split.

## Verify after split
- `cargo test -p qbz-text-utils strip_html::` — the existing test suite is
  thorough (inline formatting, br/paragraph conversion, multibyte
  preservation, entity decoding incl. malformed bare forms, Windows-1252
  quirks, idempotence, a full-pipeline real-bio-tail case) and should catch
  any wiring mistake immediately; all tests must stay green with zero
  changes to their assertions.
- `cargo build` for any crate that imports `qbz_text_utils::strip_html` or
  `qbz_text_utils::decode_html_entities` (grep callers) to confirm the
  public surface didn't shift.
