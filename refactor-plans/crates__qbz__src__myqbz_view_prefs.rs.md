# crates/qbz/src/myqbz_view_prefs.rs (200 lines)

## Summary
Per-collection view-toolbar-prefs persistence for the My QBZ detail view:
a per-user JSON map (`{collection_id: Prefs}`) with `load`/`save`/`remove`,
backed by a `LazyLock<Mutex<Option<u64>>>` active-user-id binding.

## Proposed split
This file is only modestly over budget (200 vs 130); a two-way split by
model/storage vs. tests is sufficient — no need to fragment further.

- `myqbz_view_prefs/mod.rs` (~25 lines) — module doc, `pub mod`
  declarations, `pub use` re-exports of `Prefs`, `init_for_user`, `load`,
  `save`, `remove` so `crate::myqbz_view_prefs::X` paths are unchanged.
- `myqbz_view_prefs/model.rs` (~60 lines) — `Prefs` struct + its `Default`
  impl + the `d_list`/`d_position`/`d_asc`/`d_all` default-value helpers.
- `myqbz_view_prefs/store.rs` (~95 lines) — `USER_ID` static,
  `store_path`, `read_all`, `write_all`, `init_for_user`, `load`, `save`,
  `remove`.
- `myqbz_view_prefs/tests.rs` (~35 lines) — the `#[cfg(test)] mod tests`
  block (3 tests).

## Re-export surface
`myqbz_view_prefs/mod.rs` re-exports `Prefs`, `init_for_user`, `load`,
`save`, `remove` at `crate::myqbz_view_prefs::*` — called from the My QBZ
detail-view open/toolbar-mutation handlers and from `myqbz_edit`/
`myqbz_detail` (per the module doc: "driven from `myqbz_detail` +
`myqbz_edit`") and from collection-delete cleanup (`remove` "clear on
delete"); grep those two sibling files for `myqbz_view_prefs::` call sites
before finalizing to confirm nothing else reaches in.

## Coupling / watch out
- `Prefs`'s five persisted fields (`view_mode`, `sort_by`, `sort_dir`,
  `type_filter`, `src_qobuz`, `src_local` — six fields, five logical
  slots since source is split into two bools) are a direct 1:1 mirror of
  Tauri's `collection-view-prefs.{collectionId}` JSON shape per the module
  doc — keep `model.rs`'s field names/serde defaults exactly as-is; any
  drift breaks cross-frontend compatibility for users who've used both
  Tauri and Slint builds against the same data dir.
- `searchQuery`/`selectMode` are explicitly NOT part of `Prefs` (called
  out twice in comments as "intentionally TRANSIENT") — don't accidentally
  add them during the split; this is a deliberate omission, not missing
  coverage.
- `USER_ID` (a `LazyLock<Mutex<Option<u64>>>`) is read by `store_path`
  (used by both `read_all` and `write_all`) and written only by
  `init_for_user` — keep all three in `store.rs` together since they share
  this one static; don't split `store_path` away from `USER_ID`'s
  declaration.
- The "hydrated" persist-gating flag mentioned in the module doc ("gated
  behind a `hydrated` flag so the restore is not clobbered by an early
  persist") lives in the CALLING code (`myqbz_detail`/`myqbz_edit`), not in
  this file — this file's `save`/`load` are unconditional; don't assume
  this file needs to implement that gate itself.

## Verify after split
- `cargo test -p qbz myqbz_view_prefs::` — all 3 tests green.
- `cargo check -p qbz` to confirm `myqbz_detail`/`myqbz_edit` call sites
  into `myqbz_view_prefs::{Prefs,init_for_user,load,save,remove}` still
  resolve.
- Manual smoke-test: open a My QBZ collection detail view, change view
  mode/sort/filter, navigate away and back (confirm the toolbar state
  restored), delete the collection (confirm its prefs entry is gone from
  `collection_view_prefs.json`).
