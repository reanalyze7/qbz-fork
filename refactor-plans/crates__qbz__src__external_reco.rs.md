# crates/qbz/src/external_reco.rs (945 lines)

## Summary
Discover > Recommendations tab controller: wires `qbz-external-reco` to
Slint via a `RecoCatalog` impl over `QbzCore`, manages the per-user
resolution/results cache lifecycle, and progressively builds+paints each
carousel row (artists/albums/tracks) as its builder resolves, with a
"not interested" live backfill mechanism for the two artist rails.

## Proposed split
Turn into an `external_reco/` directory:

- `external_reco/mod.rs` (~60 lines) — module doc, `CACHE_DIR` /
  `ARTIST_OVERFLOW` statics, `init_for_user`, `teardown`, `rotation_seed`,
  `pub use` re-exports of every `pub fn` from submodules.
- `external_reco/catalog.rs` (~60 lines) — lines 65-124: `CoreRecoCatalog`
  struct + its `RecoCatalog` trait impl (the QbzCore adapter).
- `external_reco/loader.rs` (~240 lines) — lines 126-249 + 251-488:
  `ensure_loaded`, `force_reload`, `spawn` (the big cache-check/build/cache-
  write orchestration), `latch_loaded`, `set_pending`, `clear_all_pending`.
  `spawn` alone is ~240 lines — if still over budget after extraction,
  further split it into `loader/build.rs` (the cold-start vs full-build
  branches, lines 380-455) vs `loader/cache.rs` (the results-cache read/write
  around it, lines 331-379 + 456-486).
- `external_reco/album_similar.rs` (~90 lines) — lines 163-242:
  `LASTFM_SIMILAR_TTL_SECS`, `load_similar_albums_seeded` (the album-page
  Last.fm "similar albums" row, independent of the Discover tab's own
  builder/cache but reusing the same `RecoCache`/`CoreRecoCatalog`).
- `external_reco/artist_rails.rs` (~170 lines) — lines 545-687:
  `artist_exclusions`, `apply_artist_rails`, `pop_backfill`,
  `apply_artist_dismissal` (the shared choke point + live "not interested"
  backfill for the two Recommended-Artist rails).
- `external_reco/apply.rs` (~230 lines) — lines 689-945: `build_and_apply_
  weeklies`, the `ArtistRow`/`AlbumRow`/`TrackRow` enums, `list_track_ids`,
  `slim_from_artist`, `slim_from_track`, `album_card`, `apply_artists`,
  `apply_tracks`, `album_row_title`, `apply_albums`, and `apply_all` (line
  536-543, move alongside — it calls these appliers). This is the
  "paint the Slint models" half.

## Re-export surface
`external_reco/mod.rs` becomes the `mod external_reco;` target. Every
currently-`pub fn` (e.g. `init_for_user`, `force_reload`, `ensure_loaded`,
`apply_artist_dismissal`, `list_track_ids`, `load_similar_albums_seeded`) must
stay reachable at `crate::external_reco::foo` via `pub use` in `mod.rs`.
`pub(crate) fn album_card` keeps its `pub(crate)` visibility on re-export
(used by other Discover-tab controllers, e.g. `reco.rs`).

## Coupling / watch out
- `CACHE_DIR` (mod.rs) is read from `loader.rs` AND `album_similar.rs` —
  keep it in `mod.rs`, both submodules `use super::CACHE_DIR`.
- `ARTIST_OVERFLOW` is written by `artist_rails.rs::apply_artist_rails` and
  read/popped by `artist_rails.rs::pop_backfill`/`apply_artist_dismissal` —
  keep all three in the same module (`artist_rails.rs`), do not split further.
- `apply_artist_rails` (artist_rails.rs) calls `apply_artists` (apply.rs) —
  a genuine cross-module call after the split; keep both `pub(crate)` (or
  `pub`) so it resolves without visibility issues.
- The `ArtistRow`/`AlbumRow`/`TrackRow` enums in `apply.rs` are referenced
  from `artist_rails.rs` (`ArtistRow::RecArtistsCommon` etc.) and from
  `loader.rs`'s build branches — either re-export them from `apply.rs` via
  `pub(crate)` or hoist them to `mod.rs` if the cross-imports get circular.
- `rotation_seed()` is used by both `loader.rs::spawn` and
  `album_similar.rs::load_similar_albums_seeded` — keep it a free fn in
  `mod.rs`.
- `RecoInputs`/`RecoCache`/builder fns come from the external `qbz_external_
  reco` crate, not this file — no internal coupling risk there, just repeat
  the `use` in every submodule that calls them.

## Verify after split
- `cargo check -p qbz` and `cargo build -p qbz` (no `#[cfg(test)]` in this
  file).
- Manually smoke-test the Discover > Recommendations tab: cold-start
  (no Last.fm/ListenBrainz) editorial fallback, warm build with both
  services connected, "Refresh now" force-reload, a "not interested" dismiss
  on both artist rails (verify backfill from the retained overflow pool),
  and the album-page "similar albums" Last.fm row.
