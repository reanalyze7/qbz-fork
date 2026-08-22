# crates/qbz-app/src/settings/favorites.rs (402 lines)

## Summary
Favorites tab-order/customization preferences: `FavoritesPreferences`
struct (+ default), a SQLite-backed `FavoritesPreferencesStore` (with
custom-icon path normalization), a session-wrapper `
FavoritesPreferencesState`, and free `create_table`/`load_preferences`/
`save_preferences` functions used directly against a `rusqlite::Connection`
(likely shared with a migration path). Has a `#[cfg(test)]` module.

## Proposed split
- `favorites/mod.rs` (~15 lines) — re-exports.
- `favorites/prefs.rs` (~30 lines) — `FavoritesPreferences` struct +
  `Default` impl (lines 10-32).
- `favorites/store.rs` (~150 lines) — `FavoritesPreferencesStore` struct +
  impl: `open_at`, `new`, `new_at`, `favorites_icon_dir`,
  `normalize_custom_icon_path`, `get_preferences`, `save_preferences`
  (lines 33-222).
- `favorites/state.rs` (~35 lines) — `FavoritesPreferencesState` struct +
  impl: `new`, `new_empty`, `init_at`, `teardown` (lines 223-259).
- `favorites/schema.rs` (~90 lines) — free functions `create_table`,
  `load_preferences`, `save_preferences` (lines 261-339) — note the name
  collision with `FavoritesPreferencesStore::save_preferences`; keep this
  module's version namespaced (`favorites::schema::save_preferences`) to
  avoid ambiguity when both are `pub use`d.
- `favorites/tests.rs` (~65 lines) — the `#[cfg(test)] mod tests` block
  (lines 340-402).

## Re-export surface
`favorites/mod.rs` must re-export `FavoritesPreferences`,
`FavoritesPreferencesStore`, `FavoritesPreferencesState` at their current
`qbz_app::settings::favorites::X` paths. A repo-wide grep for
`favorites::create_table`/`favorites::load_preferences`/
`favorites::save_preferences` found NO external callers — these three
free functions duplicate the store's own inline schema/query logic
(`open_at`'s inline `CREATE TABLE`, `get_preferences`,
`Store::save_preferences`) almost verbatim and look like dead/legacy code
kept for a bundle-import generic domain interface that never got wired,
or simply forgotten. Re-verify with a fresh grep before the real split;
if confirmed unused, consider flagging for removal in a follow-up (out of
scope for this line-count-only split — just re-export as-is for now).

## Coupling / watch out
- Two different `save_preferences` exist at different levels (the store
  method and the free function operating on a raw `Connection`) — do not
  let a `pub use schema::*;` accidentally shadow or collide with
  `store::FavoritesPreferencesStore::save_preferences` (methods don't
  collide with free functions of the same name, but grep-based callers
  might be confused — flag clearly in code comments).
- `normalize_custom_icon_path` writes into a `favorites_icon_dir()` — this
  path-handling logic should stay adjacent to `open_at`/the store, not
  move to `schema.rs`.

## Verify after split
- `cargo test -p qbz-app settings::favorites` green.
- `cargo build -p qbz-app`.
