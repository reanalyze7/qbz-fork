# crates/qbz-core/src/core.rs (2940 lines)

## Summary
The `QbzCore<A: FrontendAdapter>` orchestrator: the single frontend-agnostic API
surface (Tauri/Slint/CLI) wrapping auth, queue, search/catalog, streaming,
playback, favorites, playlists, label pages, MusicBrainz discovery/resolution,
plus free pure helper functions (blacklist filtering, search-page parsing,
playlist-duplicate math) and ~370 lines of unit tests.

## Proposed split
By domain, mirroring the file's own `// ==== Section ====` markers (already a
clean seam). All `impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A>`
blocks become **multiple `impl` blocks in different files** — legal in Rust as
long as they're in the same crate; no trait object boundary is crossed.

- `core/mod.rs` (~90 lines) — module doc, imports, `BlacklistFilter` /
  `AlbumBlacklistFilter` type aliases, the `QbzCore` struct definition, `new()`,
  and re-exports (`pub use` the free functions from `filters.rs` so
  `crate::core::album_blacklisted` etc. keep working).
- `core/filters.rs` (~180 lines) — the free functions `parse_page`,
  `pick_most_popular`, `album_blacklisted`, `track_blacklisted`,
  `discover_album_blacklisted`, `parse_search_all` (lines 44-238). Pure, no
  `QbzCore` dependency — the natural "pure computation" module.
  their unit tests move to `filters/tests.rs` or stay `#[cfg(test)]` inline here.
- `core/auth.rs` (~130 lines) — `init`, `is_api_initialized`, `try_init_api`,
  `has_session`, `login`, `login_with_token`, `set_session`, `logout` (lines
  279-471), plus `set_musicbrainz_cache`/`set_artist_vectors`/
  `clear_artist_vectors`/offline-only queue flag setters that live right above
  (these are cheap state setters, fine to keep with auth/init as "startup wiring").
- `core/queue.rs` (~330 lines) — the whole "Queue Operations" section (473-870):
  get_queue_state*, repeat/shuffle, clear/add/remove/move tracks,
  play_index/play_upcoming_at, the three `*_resolved`/`fetch_for_*` streaming
  playback helpers (play_track_resolved, fetch_for_gapless_resolved,
  fetch_for_external_stream_resolved), next/previous, peek_upcoming,
  current_track, stop-after markers, sync_current_to_id. This is the single
  biggest section — consider a further split into `queue/state.rs` (pure queue
  state ops) vs `queue/playback_resolve.rs` (the offline/network tier-walk
  functions), each comfortably under 130.
- `core/search.rs` (~90 lines) — "Search & Catalog" (872-981): search_albums/
  tracks/artists, catalog_search, search_all, get_album/get_track/get_artist.
- `core/playback.rs` (~50 lines) — "Playback Operations" (1000-1046): pause,
  resume, stop, seek, set_volume, get_playback_state, player(). All sync (no
  `.await`), thin wraps over `self.player`.
- `core/favorites.rs` (~130 lines) — "Favorites" (1048-1174): get/add/remove
  favorite, favorite_track_ids, favorite_artist_ids (the two paginating
  loops are near-identical — consider factoring a private
  `collect_favorite_ids(fav_type)` helper shared by both), set_track_favorite.
- `core/playlists.rs` (~230 lines) — "Playlists" (1175-1439 excluding labels):
  get/create/update/delete/subscribe/unsubscribe playlist, add/remove tracks,
  check_playlist_duplicates, search_playlists, get_tracks_batch, get_genres,
  discover index/playlists/tags/albums, featured albums, release watch,
  artist page/similar/suggest/dynamic-suggest, get_artist_with_albums/
  get_artist_albums/get_artist_detail/get_artist_tracks/get_releases_grid/
  get_artist_story. This section is itself large (~260 lines) — split further
  into `playlists/crud.rs` (playlist CRUD + duplicates) and
  `playlists/discover.rs` (discover/genres/featured/artist-page/suggest calls)
  if it doesn't fit 130 alone.
- `core/labels.rs` (~150 lines) — the `get_label_*` family (1626-1779):
  get_label_page/explore/albums/next_releases/awarded_releases/playlists/
  top_artists/story/list. Split into two files (`labels/browse.rs`,
  `labels/detail.rs`) if over 130.
- `core/events.rs` (~10 lines) — "Event Emission" accessors: `adapter()`,
  `client()`, `queue()` (1780-1804). Tiny; could fold into `mod.rs` instead of
  its own file if that keeps `mod.rs` under budget.
- `core/musicbrainz.rs` (~700 lines, the largest remaining chunk: 1805-2506) —
  musicbrainz_is_enabled/set_enabled, musicbrainz_resolve_artist,
  generate_playlist_suggestions, musicbrainz_get_artist_metadata,
  musicbrainz_get_artist_relationships, musicbrainz_discover_artists,
  discover_artists_by_location, musicbrainz_resolve_musician,
  musicbrainz_get_musician_appearances. MUST be split further, e.g.:
  - `musicbrainz/enable.rs` — enable/disable + resolve_artist (thin).
  - `musicbrainz/suggestions.rs` — generate_playlist_suggestions.
  - `musicbrainz/metadata.rs` — get_artist_metadata + get_artist_relationships
    (these two are the cache-read/cache-write pair, keep together).
  - `musicbrainz/discovery.rs` — musicbrainz_discover_artists +
    discover_artists_by_location (these are the two heaviest methods, ~370
    lines combined per the line numbers — 2037-2414 alone is 377 lines, so
    `discover_artists_by_location` likely needs its OWN file,
    `musicbrainz/discovery_location.rs`).
  - `musicbrainz/musician.rs` — resolve_musician + get_musician_appearances.
- `core/helpers.rs` (~60 lines) — the free functions after the `impl` block:
  `normalize_artist_name`, `shuffle_with_seed`, `compute_playlist_duplicates`
  (2508-2563). Pure, no `QbzCore` dependency.
- `core/tests.rs` (~380 lines, 2565-2940) — the entire `#[cfg(test)] mod
  tests`. Since tests reference `parse_search_all`, `album_blacklisted`,
  `track_blacklisted`, `discover_album_blacklisted`, `compute_playlist_duplicates`
  (now living in `filters.rs` / `helpers.rs`), keep `use super::*;` working via
  `core/mod.rs` re-exporting everything, OR split tests to live alongside their
  target module (`filters.rs` tests with `filters.rs`, playlist-duplicate tests
  with `helpers.rs`) — the latter is cleaner per the pure/IO split principle and
  avoids one more 380-line file.

## Re-export surface
`core/mod.rs` stays the public surface: `pub struct QbzCore`, `pub type
BlacklistFilter`/`AlbumBlacklistFilter`, and `pub use filters::{album_blacklisted,
track_blacklisted, discover_album_blacklisted}` (these three are called from
outside `qbz-core`, per their doc comments referencing search/discovery/queue
call sites). `pub(crate) use filters::parse_search_all` stays crate-private.
Every `QbzCore<A>` method across the split files is still reached as
`core::QbzCore::method()` since they're all `impl` blocks for the same type —
callers (`crates/qbz`, `crates/qbz-app`, `qbzd`) need ZERO changes.

## Coupling / watch out
- `QbzCore` is generic over `A: FrontendAdapter + Send + Sync + 'static` — every
  split-out `impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A>` block
  must repeat that exact bound or methods silently become unreachable/ambiguous.
- The queue methods almost universally end with `self.emit(CoreEvent::QueueUpdated
  {...}).await` — `emit` itself is defined in whichever file becomes `events.rs`
  (or stays in `mod.rs`); make sure it's `pub(crate)` visible to every submodule
  file (via `impl` in the same module tree, this is automatic since they're all
  `impl QbzCore` — no visibility issue, just keep it defined once).
- `queue_offline_only` (AtomicBool) is touched by both `set_queue`/`set_queue_with_order`
  (queue.rs) and `set_queue_offline_only`/`queue_is_offline_only` (wherever init/auth
  helpers land) — these three call sites must NOT diverge; keep a code comment
  cross-referencing D8.
- `musicbrainz_cache` (std Mutex, sync lock) vs `artist_vectors` (tokio Mutex,
  held across `.await`) — do not conflate these two when splitting; they have
  different locking disciplines for a reason (documented in the struct fields).
- The two favorite-id-paging loops (`favorite_track_ids` / `favorite_artist_ids`)
  are near-duplicates; a good opportunity to factor a shared private helper
  during the real split, but that's a behavior-preserving refactor beyond scope
  here — flag it, don't do it blind.
- `discover_artists_by_location` (2203-2414, ~211 lines) is itself long enough
  that splitting `musicbrainz.rs` may still leave one function over 130 lines by
  itself — that's a function-level split (not file-level) to solve when doing
  the real work; note it rather than silently ignoring the budget.

## Verify after split
- `cargo build -p qbz-core` and `cargo build --workspace` (this crate is
  depended on by `qbz`, `qbz-app`, `qbzd` — a broken re-export breaks the whole
  workspace).
- `cargo test -p qbz-core` — all tests in the (relocated) `#[cfg(test)]` blocks
  green, especially the blacklist/duplicate-detection pure-function tests.
- `cargo clippy -p qbz-core` to catch any accidentally-`pub` leaked internals or
  dead re-exports from the split.
- Smoke-test importers: `grep -rn "qbz_core::" crates/qbz crates/qbz-app
  crates/qbzd` still resolves; specifically confirm `QbzCore::new`,
  `.init()`, `.login()`, `.get_queue_state()`, `.search_all()`, and
  `.musicbrainz_discover_artists()` call sites still compile unchanged.
