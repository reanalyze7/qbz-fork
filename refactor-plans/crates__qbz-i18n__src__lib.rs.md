# crates/qbz-i18n/src/lib.rs (287 lines)

## Summary
Frontend-agnostic gettext-style translation catalog: language state (atomic
index), lazily-parsed embedded `.po` catalogs, `t`/`tn`/`t_args`/`tf`
translate functions, `{}`-placeholder substitution, and `resolve_auto` (POSIX
locale-env resolution). ~115 lines of logic, ~115 lines of tests.

## Proposed split
- `lib.rs` (~30 lines) — module declarations, `pub use` re-exports of
  `plural::PluralRule`/`po::Catalog`, `LANGS`/`CURRENT`/`CATALOGS` statics,
  `lang_index`/`catalog`/`current_catalog` (the language-state core).
- `language.rs` (~25 lines) — `set_language`, `current_language`,
  `resolve_auto` (POSIX `$LC_ALL`/`$LC_MESSAGES`/`$LANG` precedence).
- `translate.rs` (~40 lines) — `mark`, `t`, `tn`, `t_args`, `tf`,
  `substitute`.
- `tests/mod.rs` or `tests.rs` (~115 lines) — the existing `#[cfg(test)] mod
  tests` moved as-is (uses `super::*`, no changes needed beyond the module
  path).

## Re-export surface
`lib.rs` keeps `pub use translate::{t, tn, t_args, tf, mark}` and `pub use
language::{set_language, current_language, resolve_auto}` so every external
caller (`qbz_i18n::t(...)` etc.) is unaffected — it's a leaf crate with no
internal callers to break beyond its own re-exports.

## Coupling / watch-outs
- `CURRENT`/`CATALOGS` statics must stay in one file (`lib.rs`) since both
  `language.rs` and `translate.rs` read/write them — use `pub(crate)` or
  keep the accessor functions (`catalog`, `current_catalog`, `lang_index`)
  in `lib.rs` and have the other two files call them, rather than exposing
  the statics directly.
- Tests use a `Mutex<()>` lock to serialize language-mutating tests — this
  must move with the tests, unchanged.
- The `include_str!` paths for the embedded `.po` files are relative to
  `lib.rs`'s location — if `catalog()` stays in `lib.rs` this is a non-issue.

## Verify after split
`cargo test -p qbz-i18n` green; `cargo check -p qbz-ui` (or whichever crate
depends on qbz-i18n) still resolves `qbz_i18n::t`/`qbz_i18n::set_language`
etc. unchanged.
