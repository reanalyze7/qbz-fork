# crates/qbz-external-reco/src/carousels.rs (776 lines)

Per-row candidate generation for external recommendations: artist/album/
track validation pools, similar-artist rails, deep cuts, weekly playlists,
fresh releases, cold-start editorial, plus display composition (dedup +
backfill).

## Proposed split

- `carousels/mod.rs` (~40 lines) — re-export surface + shared consts
  (`DISPLAY_CAP`, `ARTIST_DISPLAY_CAP`, `PLAYLIST_CAP`,
  `VALIDATE_CONCURRENCY`, `ARTIST_SEEDS`, `SIMILAR_PER_SEED`,
  `KNOWN_ARTISTS_PER_BUILD`), `track_key`/`album_key`/`rotate_take` helpers.
- `carousels/validate_pools.rs` (~90 lines) — `validate_artist_pool`,
  `validate_album_pool`, `validate_track_pool`.
- `carousels/history.rs` (~40 lines) — `gather_history`.
- `carousels/artists.rs` (~220 lines) — `round_robin`, `similar_artist_row`,
  `build_rec_artists_common`, `build_rec_artists_recent`,
  `ArtistRailComposition`, `compose_artist_rails`, `compose_one_rail`.
- `carousels/albums.rs` (~150 lines) — `build_rec_albums`,
  `build_similar_albums_seeded`, `build_fresh_releases`,
  `build_deep_cut_albums`.
- `carousels/weekly.rs` (~120 lines) — `cached_weekly_fallback`,
  `build_weekly`.
- `carousels/editorial.rs` (~50 lines) — `build_editorial`.
- `carousels/tests.rs` (~70 lines) — existing test module (mostly tests
  `compose_artist_rails`, so it naturally pairs with `artists.rs`, but
  keep as its own file to keep the module list flat).

## Tricky coupling

- `PER_SEED_CAP` const is defined right before `round_robin`/
  `similar_artist_row` — keep with `artists.rs`.
- `validate_*_pool` fns from `validate_pools.rs` are called by almost
  every builder file (`artists.rs`, `albums.rs`, `weekly.rs`) — needs
  `use super::validate_pools::{...}` in each.
- `rotate_take` (generic helper) used everywhere — keep in `mod.rs`.

## Verify after split

`cargo build -p qbz-external-reco`, `cargo test -p qbz-external-reco
carousels::` (5 existing rail-composition tests must stay green).
