# crates/qbz-i18n/src/plural.rs (185 lines)

## Summary
Minimal gettext `Plural-Forms` header evaluator: parses the handful of
plural expressions the shipped locales actually use (en/es/de/pt/nl "!=1",
fr ">1", ja single-form, ru Slavic 3-form) into a `PluralRule`, then maps a
count `n` to a plural-form index.

## Proposed split
55 lines over budget; the file is naturally pure-logic + tests already.

- `plural/mod.rs` (~70 lines) — `Kind` enum, `PluralRule` struct + its
  `Default`/`parse`/`nplurals`/`index` impl (the public API surface).
- `plural/parse.rs` (~50 lines) — `parse_nplurals` free fn (currently at the
  bottom of the file) plus the `parse` body's expression-matching logic,
  moved out of `PluralRule::parse` into a `pub(super) fn detect_kind(&str,
  usize) -> Kind` helper so `mod.rs`'s `parse` stays a thin caller.
- `plural/tests.rs` (~70 lines, `#[cfg(test)] mod tests` content unchanged) —
  the five existing unit tests, included via `#[cfg(test)] mod tests;` from
  `mod.rs` (or kept inline in `mod.rs` if the split leaves it under budget —
  recompute after moving `parse_nplurals`/`detect_kind` out).

## Re-export surface
`plural/mod.rs` re-exports `PluralRule` (the only public type); every caller
uses `crate::plural::PluralRule` or `qbz_i18n::plural::PluralRule` —
unchanged.

## Coupling / watch out
- `Kind` is a private enum (no `pub`) — keep it `pub(crate)` at most if it
  needs to cross the `parse.rs`/`mod.rs` file boundary within the same
  module; `pub(super)` from `parse.rs` works since both are children of
  `plural`.
- The Russian-rule detection depends on a very specific substring
  (`"n%10==1&&n%100!=11"`) matched against the whitespace-stripped header —
  do not "clean up" that string during the move, it's load-bearing.
- No external crate dependents beyond `qbz_i18n`'s own `t`/translation
  lookup path — check for `PluralRule::parse` call sites before/after.

## Verify after split
- `cargo test -p qbz-i18n plural` (all 5 existing tests must stay green,
  unchanged assertions)
- `cargo check -p qbz-i18n`
- Grep for `PluralRule` importers outside `qbz-i18n` (likely just the
  crate's own translation-lookup module) to confirm no path changed.
