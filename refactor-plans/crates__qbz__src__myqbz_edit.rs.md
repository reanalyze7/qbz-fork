# crates/qbz/src/myqbz_edit.rs (344 lines)

## Summary
Wires the My QBZ collection-detail hero overflow menu (Rename / Description /
Play-mode toggle / Convert kind / Delete / bulk-remove) to `qbz_mixtape::repo`
setters via `library_db::with_db`, reloading the detail view and toasting
outcomes after each mutation.

## Proposed split
- `myqbz_edit/mod.rs` (~35 lines) — module doc (lines 1-22), the
  `with_repo` DB-write helper (lines 35-44), `pub use` re-export of every
  public entry point from `actions.rs`.
- `myqbz_edit/reload.rs` (~20 lines) — the `reload` helper (lines 51-63):
  re-navigates the open detail view after a mutation.
- `myqbz_edit/actions.rs` (~230 lines) — the public entry points: `rename`,
  `set_description`, `toggle_play_mode`, `convert_kind`, `delete`,
  `remove_selected`. These are the "public API surface" callers use (the
  hero overflow menu + modal submit handlers) — kept together since they're
  all short, similarly-shaped `spawn` + `spawn_blocking` + `finish`/`toast`
  wrappers around a single repo call, and splitting them further (e.g. one
  file per action) would fragment 6 near-identical ~35-line functions across
  6 files for no cohesion benefit. If a stricter per-action split is
  preferred, an alternative is `actions/rename.rs`, `actions/description.rs`,
  `actions/play_mode.rs`, `actions/kind.rs`, `actions/delete.rs`,
  `actions/remove_selected.rs` (~35-55 lines each) re-exported from
  `actions/mod.rs`.
- `myqbz_edit/modal.rs` (~45 lines) — the modal-state helpers: `finish`,
  `close_modal`, `set_busy` (lines 300-344) — the shared "on success/failure"
  bookkeeping every action in `actions.rs` calls.

## Re-export surface
`myqbz_edit/mod.rs` re-exports `rename`, `set_description`,
`toggle_play_mode`, `convert_kind`, `delete`, `remove_selected` at
`crate::myqbz_edit::*` — existing callers (the hero overflow menu Slint
callback wiring in the AppShell controller) keep calling
`crate::myqbz_edit::rename(...)` etc. unchanged.

## Coupling / watch out
- Every action in `actions.rs` calls `finish`, `close_modal`, or `set_busy`
  from `modal.rs`, plus `reload` from `reload.rs`, plus `with_repo` from
  `mod.rs` — all four need `pub(super)` or `pub(crate)` visibility once
  they're not co-located in one file (currently private `fn`s).
  `convert_kind` and `delete` inline their own success/error handling instead
  of calling `finish` (they toast directly and call `reload`/navigate
  themselves) — verify these two still compile against the moved
  `reload`/toast helpers after the split.
- `with_repo` (in `mod.rs`) is the single DB-access chokepoint — every action
  goes through it via `spawn_blocking(move || with_repo(|conn| ...))`; do not
  duplicate this helper when splitting `actions.rs` further, always import
  it from `mod.rs`.
- `delete` additionally calls `crate::myqbz_view_prefs::remove(&id)` and
  `NavState::invoke_request_back()` (not just `reload`) — this is a
  documented deliberate divergence from the other actions (it navigates back
  instead of reloading in place since the collection itself is gone) — keep
  that special-casing intact wherever `delete` ends up.
- `remove_selected` sorts positions descending before the loop specifically
  so the repo's position-compaction doesn't invalidate later positions in
  the same batch — this ordering must survive the split verbatim (it's a
  correctness requirement, not a style choice).

## Verify after split
- `cargo check -p qbz` — no unit tests exist in this file today; confirm
  whether `qbz_mixtape::repo` has its own test coverage for the underlying
  setters (that coverage is unaffected by this split either way).
- Grep `crate::myqbz_edit::` across `crates/qbz/src/` for the hero overflow
  menu wiring and confirm every call site (`rename`, `set_description`,
  `toggle_play_mode`, `convert_kind`, `delete`, `remove_selected`) still
  resolves.
- Smoke-test in the running app: rename a collection, edit its description,
  toggle play mode, convert mixtape<->collection, bulk-remove items, and
  delete a collection — verify each still reloads/toasts/navigates
  correctly.
