# crates/qbz/src/fav_cache.rs (296 lines)

## Summary
Process-wide in-memory caches (favorite tracks, favorite albums, followed
artists) backed by a per-user `FavoritesCacheStore`, with disk-first
seeding on session activation and optimistic toggle + disk mirroring.

## Proposed split
The file already has explicit `// ====` banner sections per entity type —
split exactly along those, one file per entity plus lifecycle:

- `fav_cache/mod.rs` (~35 lines) — the four `static`s (`FAVORITES`,
  `FAV_ALBUMS`, `FAV_ARTISTS`, `STORE`), `pub use` of submodules.
- `fav_cache/lifecycle.rs` (~75 lines) — `init_for_user`, `teardown` (seeds/
  clears all three sets together).
- `fav_cache/tracks.rs` (~65 lines) — `set_all`, `is_favorite`, `all`,
  `contains`, `set` (the favorite-tracks API, pre-existing before the
  albums/artists sections were added).
- `fav_cache/albums.rs` (~50 lines) — `is_album_favorite`, `set_all_albums`,
  `set_album`.
- `fav_cache/artists.rs` (~55 lines) — `is_artist_favorite`, `all_artists`,
  `set_all_artists`, `set_artist`.

## Re-export surface
`fav_cache/mod.rs` stays the `mod fav_cache;` target with the shared statics
defined there; every submodule does `use super::{FAVORITES, FAV_ALBUMS,
FAV_ARTISTS, STORE};` as needed. All public fns re-exported via `pub use
lifecycle::*; pub use tracks::*; pub use albums::*; pub use artists::*;` so
call sites across the app (`crate::fav_cache::is_favorite(...)` etc.) are
unaffected.

## Coupling / watch out
- `init_for_user` and `teardown` (lifecycle.rs) touch ALL FOUR statics —
  this is the one file that can't be per-entity; keep it as its own module
  rather than duplicating logic into each entity file.
- Every mutation function follows the same two-step pattern (update the
  in-memory `RwLock`, then best-effort mirror to `STORE`) — preserve this
  ordering exactly (memory first, then disk) in each split-out file, it's
  what makes UI toggles feel instant while disk write is fire-and-forget.
- `STORE: Mutex<Option<FavoritesCacheStore>>` is a single shared store
  used by all three entity types (`sync_favorite_tracks`,
  `sync_favorite_albums`, `sync_favorite_artists`, etc. are all methods on
  the same store type) — must stay a single instance in `mod.rs`, not
  duplicated per file.
- No `#[cfg(test)]` in this file — flag the absence of test coverage for
  the split; verification is compile + manual only.

## Verify after split
- `cargo build -p qbz`.
- Smoke-test: favorite/unfavorite a track, an album, and follow/unfollow an
  artist; restart the app and confirm all three persisted correctly
  (disk-first seeding still works offline).
