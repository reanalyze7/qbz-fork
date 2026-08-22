# `crates/qbz-theme/src/id.rs` (292 lines)

## Summary
Defines the stable `ThemeId` enum (36 variants) and `ThemeCategory` enum that
together form the theme *identity* layer: persisted slug <-> enum roundtrip,
human display names (proper-noun data, not i18n), category grouping used by
the Settings list, and the `default_id()`/`default_slug()` accessors. This is
the identity/metadata layer that `registry.rs` (and `custom.rs`) map into
fully-materialized `ThemeColors`.

## Proposed split

- `id/mod.rs` (~30 lines) — re-exports `ThemeId`, `ThemeCategory`, `ALL`,
  `default_slug`; hosts the module doc comment. This becomes the public API
  surface — `lib.rs` keeps `mod id;` and `pub use id::{default_slug,
  ThemeCategory, ThemeId, ALL};` completely unchanged (this file already sits
  behind that boundary, so no importer outside the crate needs to change).
- `id/category.rs` (~20 lines) — `ThemeCategory` enum + its `slug()` impl.
- `id/theme_id.rs` (~55 lines) — the `ThemeId` enum definition itself and the
  `ALL: &[ThemeId]` const array (kept together since `ALL` must be
  exhaustively kept in sync with the enum variants — reviewers should diff
  these as one unit).
- `id/slug.rs` (~45 lines) — `impl ThemeId` block containing `slug()` and
  `from_slug()` (the persisted-string roundtrip pair — kept together since
  they're tested as a pair).
- `id/display_name.rs` (~45 lines) — `impl ThemeId` block containing just
  `display_name()` (the large proper-noun match arm list; isolated because
  it's the block most likely to grow with new themes, and is pure
  presentation data with no logic).
- `id/category_map.rs` (~25 lines) — `impl ThemeId` block containing
  `category()` (the Core/Dark/Light/Accessibility grouping match, including
  the Frost/Langley/Alucard placement notes).
- `id/meta.rs` (~15 lines) — `impl ThemeId` block containing `default_id()`
  and `is_implemented()`, plus the free function `default_slug()`.
- `id/tests.rs` (~40 lines) — the existing `#[cfg(test)] mod tests` block
  (`slug_roundtrip_all`, `slugs_are_unique`, `default_is_oled`,
  `unknown_slug_is_none`, `p1_themes_implemented`), moved verbatim.

Rust allows splitting a single `impl ThemeId` block across multiple files
(each file just writes `impl ThemeId { ... }` again), so `theme_id.rs`,
`slug.rs`, `display_name.rs`, `category_map.rs`, and `meta.rs` are independent
`impl` blocks re-declared per file — no trait needed, this is normal Rust.

## Public API / re-export surface
`id/mod.rs` is the module's public surface: `pub use theme_id::{ThemeId, ALL};
pub use category::ThemeCategory; pub use meta::default_slug;` (the two
`impl ThemeId` files just need `use super::theme_id::ThemeId;` — no re-export
needed for impl blocks). Outside the crate, `lib.rs`'s existing `pub use
id::{...}` line does not need to change at all.

## Coupling / watch-outs
- `ALL` (in `theme_id.rs`) must stay in the same file as the `ThemeId` enum
  definition or immediately adjacent — it's manually kept in sync with the
  variant list and both `registry.rs` tests (`ALL.iter()`) and `id.rs`'s own
  tests rely on it being exhaustive.
- `slug()` and `from_slug()` must stay paired (roundtrip tested together);
  don't split them into different files.
- `category()` has explicit comments about non-obvious placements (Frost /
  Langley are "registered light, visually dark"; Alucard is genuinely light
  but grouped under Tauri's "Dark" bucket) — preserve these comments verbatim
  when moving, they're load-bearing documentation for future theme additions.
- `registry.rs` and `custom.rs` both do `use crate::id::ThemeId;` — as long as
  `ThemeId` is re-exported from `id/mod.rs`, these imports keep working
  unchanged.
- Test module currently does `use super::*;` — after the split it should do
  `use super::super::{ThemeId, ALL};` or similar; keep all 5 tests in one
  `tests.rs` file since they're small and share no complex fixtures.

## Verification after the real split
- `cargo build -p qbz-theme` compiles with no changes needed in `registry.rs`,
  `custom.rs`, or `lib.rs`.
- `cargo test -p qbz-theme` — all 5 existing tests in `id::tests` still pass,
  especially `slug_roundtrip_all` and `slugs_are_unique` (they iterate `ALL`).
- `cargo doc -p qbz-theme` builds cleanly (module doc comments moved intact).
- Smoke-test importers: grep the workspace for `qbz_theme::ThemeId`,
  `qbz_theme::ALL`, `qbz_theme::default_slug`, `qbz_theme::ThemeCategory` to
  confirm none of those call sites need updating.
