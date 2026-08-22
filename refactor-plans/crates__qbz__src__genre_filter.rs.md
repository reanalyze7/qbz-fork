# crates/qbz/src/genre_filter.rs (485 lines)

## Summary
Filter-by-genre controller: loads the parent/child genre tree, owns a
per-context (discover/favorites) selection with JSON persistence, builds the
popup's flattened tree rows, and pushes state into the `GenreFilterState`
Slint global.

## Proposed split
By domain (persistence / async loaders / tree-building / mutation), mirroring
`discover_prefs.rs`'s pattern:

- `genre_filter/mod.rs` (~55 lines) — module doc, `GenreItem`, `Persisted`,
  `State` struct + its 3 methods (`cur_mut`/`is_selected`/`cur_len`),
  the `STATE` static, re-exports.
- `genre_filter/persistence.rs` (~40 lines) — `store_path`, `load_persisted`,
  `save_persisted` (pure IO, no Slint dependency — easy to unit test with a
  temp dir).
- `genre_filter/context.rs` (~55 lines) — `set_context`, `current_context`,
  `selected_ids`, `selected_ids_for`, `selected_names`,
  `collect_descendants` (the per-context read/query surface).
- `genre_filter/loaders.rs` (~90 lines) — `children_loaded`, `store_children`,
  `load_parents`, `load_children`, `child_ids`, `load_all_parent_children`,
  `load_descendants` (the async network-backed loaders).
- `genre_filter/tree.rs` (~85 lines) — `tree_row`, `build_tree_rows`,
  `toggle_expand`, `set_search` (tree flattening + expand/search state —
  pure-ish, `State` mutation only, no IO/network).
- `genre_filter/apply.rs` (~30 lines) — `apply_state` (pushes into the
  `GenreFilterState` Slint global — the one function that touches
  `AppWindow`).
- `genre_filter/mutations.rs` (~45 lines) — `toggle`, `clear`, `set_remember`
  (selection mutation + persist trigger).

## Re-export surface
`genre_filter/mod.rs` re-exports every current `pub fn`/type
(`set_context`, `current_context`, `selected_ids`, `selected_ids_for`,
`selected_names`, `children_loaded`, `load_parents`, `load_children`,
`load_all_parent_children`, `load_descendants`, `toggle_expand`,
`set_search`, `apply_state`, `toggle`, `clear`, `set_remember`) at
`crate::genre_filter::*`, so every call site elsewhere in `qbz` (the
Discover/Favorites genre popup wiring) needs no changes.

## Coupling / watch out
- The `STATE: LazyLock<Mutex<State>>` static is the one shared piece of
  state EVERY submodule above touches — must live in `mod.rs` (or a
  dedicated `state.rs`) as `pub(super)` so `persistence.rs`, `context.rs`,
  `loaders.rs`, `tree.rs`, `apply.rs`, `mutations.rs` can all `use
  super::STATE`.
- `toggle`/`clear`/`set_remember` all follow the same pattern (mutate under
  lock -> clone selected+remember -> drop lock -> `save_persisted`) — keep
  that lock-then-drop-then-persist ordering intact; do not accidentally
  call `save_persisted` while still holding the mutex (would be a
  same-thread deadlock only if `save_persisted` ever re-locked `STATE`,
  which it doesn't today, but worth a comment).
- `selected_names` recurses through `collect_descendants` against
  `s.children` — that HashMap is populated by `loaders.rs`'s
  `store_children`, so `context.rs` has a real data dependency on
  `loaders.rs` having run first; not a compile-time coupling, just a
  runtime ordering worth a one-line comment in the split.
- "discover" vs "favorites" context strings are stringly-typed with no
  shared const — consider (separately from this split, if the owner wants)
  promoting them to an enum, but that's out of scope for a pure
  file-split plan.

## Verify after split
- `cargo check -p qbz`.
- Manual smoke-test: open the genre filter popup on both Discover and
  Favorites, select genres independently on each, close and reopen the app,
  confirm the "Remember selection" persistence still round-trips per
  context.
