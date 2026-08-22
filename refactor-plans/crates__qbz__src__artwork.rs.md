# crates/qbz/src/artwork.rs (1666 lines)

## Summary
The album/artist/playlist artwork pipeline for the Slint app: a giant
`ArtworkTarget` enum naming every card/row/mosaic slot in the whole UI, job
dispatch (`spawn_loads`/`spawn_local_loads`/`spawn_search_loads`), a shared
disk + decoded-pixel cache, fetch/decode helpers, and a single huge
`apply_artwork` match that writes decoded pixels back onto the right Slint
model row for ~90 target variants.

## Proposed split
Convert `artwork.rs` into `artwork/mod.rs` + siblings (this crate's modules
are flat `.rs` files declared from `main.rs`, so this becomes the first
directory-backed module — same pattern already used in `qbz-theme/src/auto/`).

- `artwork/mod.rs` (~40 lines) — module doc, `pub use` re-exports of every
  public item from the submodules below, so `crate::artwork::X` paths used
  throughout the ~39 other files that reference `artwork::` do not change.
- `artwork/target.rs` (~300 lines) — the `ArtworkTarget` enum (lines 52-296)
  and its `decode_size()` impl (298-332). Pure data + a pure function; no
  I/O. This alone is still borderline over 130 — split further by UI area if
  needed, e.g. `target/discover_home.rs` vs `target/library_favorites.rs` vs
  `target/artist_label.rs`, re-exported from `target/mod.rs`. Simpler
  alternative: keep as one file and accept it as the one deliberate
  over-budget exception (a pure enum, not logic) — flag this choice for
  reviewer sign-off rather than silently splitting an enum across files that
  all need every variant visible together for `apply_artwork`'s match arm.
- `artwork/cache.rs` (~130 lines) — `ImageCache` type, `open_cache`,
  `open_cache_blocking`, `SHARED_CACHE`/`set_shared_cache`/`shared_cache`,
  `cached_path_for`, `cached_file_url_for`, `spawn_evict`, `MAX_CACHE_BYTES`.
  The disk-cache surface, I/O-flavored.
- `artwork/decode.rs` (~200 lines) — `pixels_to_image`, `decode_rgba`,
  `DecodedPixels`, `DecodedCache` struct + `DECODED_PIXEL_CACHE`,
  `decoded_pixels`/`store_decoded`, `DECODED_CACHE_CAP`/`DECODED_CACHE_BUDGET`,
  `decode_local_pixels`, `load_local_cover`, `header_tint`. The decode +
  in-memory LRU cache — "pure computation over bytes" module.
- `artwork/fetch.rs` (~110 lines) — `HTTP` client, `HTTP_TIMEOUT`,
  `fetch_cached_http`, `fetch_and_decode_ref`, `fetch_and_decode`. The
  network/disk I/O layer that calls into `decode.rs` and `cache.rs`.
- `artwork/jobs.rs` (~130 lines) — `ArtworkJob` struct, `pinned_artwork_jobs`,
  `spawn_loads`, `spawn_local_loads`, `spawn_search_loads`, `MAX_CONCURRENT`,
  `set_ui_scale_factor`/`scaled_decode`/`UI_SCALE_FACTOR`/`DECODE_SIZE`. The
  job-dispatch layer (semaphore-bounded spawns).
- `artwork/apply.rs` split by UI section, since the single `apply_artwork`
  match (lines 833-1665, ~830 lines) is the single biggest offender:
  - `apply/home_discover.rs` — Popular/Recent/RecentAlbum(s)/MostPlayed/
    HomeFavorite/HomeMostPlayed/HomeReleaseWatch/HomeTopArtist/
    HomePlaylistCover/PlaylistBrowseCover/DiscoverSectionAlbum/
    DiscoverBrowseAlbum arms.
  - `apply/search_immersive.rs` — SearchAlbum/Track/Artist/PlaylistCover/
    MostPopular/SidebarPlaylistCover/CortinillaRow/ImmersiveSearchRow arms
    (these share the URL-match late-arrival-guard idiom — keep together).
  - `apply/artist_label.rs` — every ArtistX/LabelX/MusicianAppearance arm.
  - `apply/library_local.rs` — LibraryAllCover/LocalX/BlacklistAlbum arms.
  - `apply/favorites_foryou.rs` — FavoriteX/ForYouX/ExtRecoX arms.
  - `apply/mix_playlist_myqbz.rs` — MixTrack/PlaylistTrack/PlaylistCover/
    PmPlaylistCover/PmTreeCover/MyQbzX/SuggestionX/PlaylistSuggestionCover/
    PinnedCard arms.
  - `apply/mod.rs` — the top-level `apply_artwork(window, target, url,
    pixels, width, height)` function that builds the `slint::Image` once
    (lines 842-850) and then dispatches `match target { ... }` to a thin
    per-arm helper function `fn apply_<name>(window, ..)` defined in each of
    the sibling files above. This requires refactoring the giant single
    `match` into named helper-function calls — a real (if mechanical)
    behavior-preserving refactor, not just a cut-and-paste; note this as the
    highest-effort item in the split.

## Re-export surface
`artwork/mod.rs` becomes the single public surface: `pub use target::*;
pub use cache::*; pub use decode::*; pub use fetch::*; pub use jobs::*; pub
use apply::apply_artwork;` (apply_artwork is currently private `fn`, stays
private — only called from within `artwork/jobs.rs`, so it can stay
`pub(super)` or `pub(crate)`). `main.rs`'s `mod artwork;` line needs zero
changes since Rust resolves `artwork/mod.rs` identically to `artwork.rs`.

## Coupling / watch out
- `apply_artwork` is called from 3 different spawn functions (`jobs.rs`) —
  make sure the function stays `pub(crate)` visible to `jobs.rs` after the
  split (same crate tree, so this is automatic via `pub(super)`/`pub(crate)`
  on `apply::apply_artwork`).
- `ArtworkTarget` variants carry data (ids, gens, slots) that OTHER modules
  pattern-match on directly (e.g. `crate::favorites::album_artwork_job_done`,
  `crate::local_library::album_artwork_job_done` are called from
  `jobs.rs`/`apply.rs` when a job fails/succeeds) — these cross-module calls
  must keep resolving; they are all `crate::` absolute paths so unaffected
  by which file `ArtworkTarget` physically lives in.
- `DECODED_PIXEL_CACHE` (in `decode.rs`) and `HTTP`/`fetch_cached_http` (in
  `fetch.rs`) are both process-wide statics — splitting into separate files
  is safe (statics are file-location-independent) but keep them each defined
  exactly once; do not accidentally duplicate the `LazyLock` static during
  the cut.
- `apply_artwork`'s `PinnedCard` arm (bottom of the match) references
  `crate::immersive::dominant_cover_color(pixels, width, height)` using the
  OUTER `pixels`/`width`/`height` params, not the local `image` — verify this
  exact variable capture survives being moved into a helper function; a
  naive per-arm split must still receive `pixels: &[u8], width: u32, height:
  u32` alongside `image: slint::Image`.
- `card.url1..url9` / `cover1..cover9` mosaic slot conventions
  (`MyQbzMixtapeCover`/`MyQbzCollectionCover`) are shared with
  `crate::myqbz::set_mosaic_cover` — do not diverge the slot-index mapping
  between the two files during the split.

## Verify after split
- `cargo build -p qbz` (this crate's main binary) — artwork.rs is imported by
  ~39 other files in `crates/qbz/src`, so a broken re-export is a wide break.
- `cargo clippy -p qbz` for accidentally-`pub` leaks or now-dead code paths.
- `grep -rn "artwork::" crates/qbz/src | grep -v artwork/` still resolves
  identically (import paths are `crate::artwork::Foo`, location-independent).
- Manual/smoke run: open the app, scroll Home/Discover/Search/Library/My QBZ
  tabs and confirm covers still paint (the apply-match split is the
  highest-risk piece — a misrouted arm silently paints nothing rather than
  crashing).
