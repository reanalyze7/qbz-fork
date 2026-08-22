# crates/qbz/src/foryou.rs (966 lines)

## Summary
Discover > For You tab controller: fetches ~9 personalized sections
(Release Watch, Recently Played, Top Artists, Artists to Follow, Rediscover,
Favorite/Most-Played/More-From-Library albums, Spotlight) concurrently,
applies each to `ForYouState` the moment it resolves, then latches
`loaded = true` once every branch finishes.

## Proposed split
By domain, following the file's own banner-comment sections:

- `foryou/mod.rs` (~40 lines) — module doc, imports, `ARTIST_SEEDS` /
  `SIMILAR_PER_SEED` / `FOLLOW_MAX` constants, `pub use` of every symbol the
  submodules below expose, and `reset_loading()` (the one tiny orchestrator
  entry point besides `spawn_for_you`).
- `foryou/models.rs` (~50 lines) — `SpotlightData`, `AlbumCard`,
  `TrackSlim`, `ArtistSlim` plain data structs plus `map_album()` /
  `map_artist()`.
- `foryou/fetch.rs` (~150 lines) — the network fetch helpers:
  `fetch_release_watch`, `fetch_fav_artists`, `fetch_fav_albums`,
  `fetch_suggest`, `fetch_to_follow` (all `async fn`s that hit
  `runtime.core()`).
- `foryou/spotlight.rs` (~120 lines) — `load_spotlight()` alone (it's a
  self-contained ~110-line async fn with its own rotation/album-grouping
  logic; giving it its own file keeps `fetch.rs` under budget).
- `foryou/build.rs` (~90 lines) — the pure (no-network) builders:
  `recent_album_cards`, `recent_track_slims`, `top_artist_slims`,
  `top_artist_cards` (async wrapper), `build_rediscover`, `order_by_score`,
  `build_favorite_albums`, `favorite_album_cards` (async wrapper),
  `most_played_album_cards`.
- `foryou/mappers.rs` (~65 lines) — the Slint model mappers: `album_items`,
  `artist_items`, `section`.
- `foryou/jobs.rs` (~60 lines) — artwork job builders: `album_jobs`,
  `artist_jobs`, `track_jobs`, `spotlight_jobs`.
- `foryou/apply.rs` (~140 lines, still slightly over — split further if
  needed into `apply_sections.rs` and `apply_misc.rs`) — the per-section
  `apply_*` functions: `apply_recent`, `apply_release_watch`,
  `apply_top_artists`, `apply_to_follow`, `apply_rediscover`,
  `apply_favorite_albums`, `apply_most_played_albums`,
  `apply_more_from_library`, `apply_spotlight`.
- `foryou/orchestrator.rs` (~175 lines) — `spawn_for_you()`, the big
  concurrent-branch orchestrator, plus its doc comment describing the
  dependency layers. This is the trickiest piece to extract cleanly (see
  coupling note) so keep it isolated in its own file rather than folding it
  into `mod.rs`.

## Re-export surface
`foryou/mod.rs` is the target of the existing `mod foryou;` (or `pub mod
foryou;`) declaration in `crates/qbz/src/main.rs` (or lib root). Every
currently-`pub(crate)`/`pub` function reachable as `crate::foryou::foo` —
notably `fetch_release_watch`, `top_artist_cards`, `favorite_album_cards`,
`artist_items`, `section`, `most_played_album_cards`, `reset_loading`,
`spawn_for_you` (several explicitly marked `pub(crate) since #566` because
`home.rs` reuses them) — must stay reachable at that same path via `pub
use fetch::*; pub use build::*; pub use mappers::*;` etc. in `mod.rs`.
`home.rs`'s existing `crate::foryou::{fetch_release_watch, top_artist_cards,
favorite_album_cards, artist_items, section}` calls are the main risk here —
double check those five names resolve post-split.

## Coupling / watch out
- **Shared with `home.rs`** (marked `pub(crate) since #566` in comments):
  `fetch_release_watch`, `top_artist_cards`, `favorite_album_cards`,
  `artist_items`, `section`. These MUST end up `pub(crate)` (not just
  `pub(super)`) in whichever submodule owns them, since `home.rs` is a
  sibling module, not a child of `foryou`.
- `spawn_for_you`'s orchestrator builds several `Pin<Box<dyn Future<Output
  = ()> + Send>>` branches inline, each closing over `runtime.clone()`,
  `weak.clone()`, `image_cache.clone()` — these closures call straight into
  `fetch_*`/`apply_*`/`load_spotlight` from other proposed submodules, so
  `orchestrator.rs` needs `use super::{fetch::*, apply::*, spotlight::*,
  build::*};` (or crate-qualified paths). Don't let a submodule boundary
  break these closures' captures.
- `has_recents_seed` / `recents_seed` / `recents_seed_title` are computed in
  Layer 0 and threaded into TWO different branches (`albums_branch` as a
  fallback-trigger condition, `suggest_branch` as the primary path) — if
  `orchestrator.rs` is split further, keep this cross-branch data flow in
  one file; splitting it would require passing extra parameters.
- `crate::artist_blacklist`, `crate::reco`, `crate::recently`,
  `crate::fav_cache`, `crate::pinned`, `crate::artwork` are cross-cutting
  deps used from `fetch.rs`, `spotlight.rs`, `build.rs`, `mappers.rs`,
  `apply.rs` — re-import in each rather than funneling through `mod.rs`.

## Verify after split
- `cargo check -p qbz` and `cargo build -p qbz` (Slint codegen must still
  see `AppWindow`/`ForYouState`/`DiscoverSection`/`AlbumCardItem`/`SlimItem`
  imports resolve correctly from wherever they land).
- Grep `crate::foryou::` across the whole `qbz` crate (especially
  `home.rs` and `main.rs`) after the split and confirm every call site still
  compiles against the new module layout.
- No `#[cfg(test)]` block exists in this file today, so there's no unit
  test to keep green — smoke-test the For You tab manually (open app,
  confirm all ~9 rails populate, confirm Home's shared rails — Top Artists /
  Library Albums / Release Watch — still render identically since they
  share these functions).
