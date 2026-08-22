# crates/qbz-i18n/src/po.rs (301 lines)

## 1. Summary
A minimal hand-rolled gettext `.po` parser producing a `Catalog` (msgid ->
msgstr / plural forms), handling multi-line continuations, comments, and
escape sequences, keyed by `msgid` (not `msgctxt`) — plus its unit test
module.

## 2. Proposed module split
| New file | Owns | ~lines |
|---|---|---|
| `po/mod.rs` | Module decls + re-export; the module doc comment | ~15 |
| `po/catalog.rs` | The `Catalog` struct + its `impl` (`parse`, `lang`, `plural_rule`, `nplurals`, `get`, `get_plural`) — the public API surface | ~120 |
| `po/parser.rs` | The line-parsing internals: `Field` enum, `Entry` struct, `flush()`, `parse_quoted()` — everything `Catalog::parse` delegates to | ~110 |
| `po/tests.rs` | The `#[cfg(test)] mod tests` block (6 tests) | ~75 |

`Catalog::parse`'s body (lines 27–106) is the one method that needs to move:
it becomes a thin wrapper that drives the line loop, calling into
`parser::flush`/`parser::parse_quoted` and building `Field`/`Entry` from
`parser.rs`, then constructs `Catalog` at the end. This is a pure
data-model (`catalog.rs`) vs. parsing-engine (`parser.rs`) split — closely
mirrors the pure/IO principle even though there's no I/O here (it's all pure
computation, just two distinct pure concerns: "what a catalog IS" vs. "how a
.po file's lines get turned into one").

## 3. Re-export / public API surface
`po/mod.rs` re-exports the single public type:

```rust
mod catalog;
mod parser;
#[cfg(test)]
mod tests;

pub use catalog::Catalog;
```

Every caller doing `use crate::po::Catalog;` (from `qbz-i18n`'s catalog
loader / the `t`/`tf` lookup functions) keeps working unchanged — `Field`,
`Entry`, `flush`, `parse_quoted` are all internal (`pub(super)` at most) and
were never part of the public surface.

## 4. Tricky coupling / shared-state to watch out for
- `Catalog::parse`'s loop directly matches on `Field`/mutates `Entry` fields
  (`cur.msgid`, `cur.last`, etc.) — when split, `catalog.rs` needs
  `use super::parser::{Entry, Field, flush, parse_quoted};` and the loop body
  itself should probably stay in `catalog.rs` (it's the `impl Catalog::parse`
  method) rather than being extracted into `parser.rs`, since it's the one
  place that owns building the final `singular`/`plural`/`plural_rule` maps.
  Only the smaller pure helpers (`flush`, `parse_quoted`, `Field`, `Entry`)
  move to `parser.rs`.
- `flush()` takes `&mut PluralRule` and calls `PluralRule::parse` on the
  header's `Plural-Forms:` line — this ties `parser.rs` to
  `crate::plural::PluralRule`; keep that `use` in `parser.rs`.
- The header-entry special case (empty `msgid` carries metadata in its
  `msgstr`, specifically `Plural-Forms:`) is handled entirely inside
  `flush()` — do not duplicate this logic in `catalog.rs`.
- Test module currently uses `super::*` — after the split it needs
  `use super::super::catalog::Catalog;` or (simpler) move `tests.rs` to
  import `crate::po::Catalog` directly, since the tests only exercise the
  public `Catalog` API and don't touch `parser.rs` internals directly.

## 5. What to verify after the real split
- `cargo test -p qbz-i18n po::` — all 6 tests green (`parses_singular`,
  `empty_msgstr_is_none`, `missing_msgid_is_none`, `parses_plural_forms`,
  `reads_nplurals_from_header`, `handles_multiline_continuation_and_escapes`).
- `cargo build -p qbz-i18n` and grep the workspace for `po::Catalog` /
  `use ... ::po::` to confirm no external crate reaches into `po::parser` or
  `po::catalog` directly (it shouldn't, since only `Catalog` was ever `pub`).
- Smoke-test: run the app with a non-English locale configured and confirm
  translated strings + a plural string (e.g. "N tracks") still render
  correctly, since this parser feeds the entire i18n catalog at startup.
