# crates/qbz/src/album.rs (1055 lines)

## Summary
Album-detail controller: fetches a full album via `QbzCore`, maps the raw
`Album`/`Track` catalog models to plain `Send` data (`AlbumData`/`TrackData`) on
a worker thread, applies it to the `AlbumState` Slint global on the UI thread, and
also owns the "more from artist" / "listening suggestions" / Last.fm-suggestions
polish carousels, track-search filtering, multi-select, and header-artwork
application.

## Proposed split
By responsibility (data mapping ↔ apply/render ↔ ancillary carousels ↔
multi-select), keeping `album.rs` itself as the re-export surface:

- `album/mod.rs` (~40 lines) — module doc, `thread_local!` (`FULL_TRACKS`,
  `PLAY_TRACKS`), `mod` declarations, `pub use` of everything the rest of the
  crate currently imports from `crate::album::*` (`AlbumData`, `TrackData`,
  `ArtistCreditData`, `load_album`, `apply_album`, `MoreFromArtist`,
  `Suggestions`, `load_more_from_artist`, `apply_more_from_artist`,
  `load_suggestions`, `apply_suggestions`, `apply_lastfm_suggestions`,
  `filter_tracks`, `reset_album`, `set_multi_select`, `recount_selected`,
  `select_all`, `clear_selection`, `selected_ids`, `selected_play_tracks`,
  `disc_play_tracks`, `apply_artwork`).
- `album/data.rs` (~90 lines) — `ArtistCreditData`, `AlbumData`, `TrackData`
  struct definitions (plain data, no logic).
- `album/map.rs` (~230 lines) — `load_album`, `format_release_date`,
  `credit_role`, `build_credits`, `map_album`, `lastfm_segment`,
  `truncate_words`, `map_track`, `tier`, `mmss`, `format_duration` (all pure
  mapping/formatting, the "computation" half of the file).
- `album/apply.rs` (~210 lines) — `apply_album` (the big Slint-global writer) and
  `apply_artwork`. This is the "render" half; reads `FULL_TRACKS`/`PLAY_TRACKS`
  thread-locals declared in `mod.rs`.
- `album/carousels.rs` (~190 lines) — `MoreFromArtist`, `Suggestions`,
  `load_more_from_artist`, `apply_more_from_artist`, `load_suggestions`,
  `apply_suggestions`, `apply_lastfm_suggestions`, `MORE_FROM_ARTIST_MAX` (the
  "polish carousels" section, already delimited by its own `// ====` header
  comment in the original file).
- `album/selection.rs` (~110 lines) — `set_multi_select`, `recount_selected`,
  `select_all`, `clear_selection`, `selected_ids`, `selected_play_tracks`,
  `disc_play_tracks` (the "Multi-select" section, also already delimited).
- `album/reset.rs` (~40 lines) — `reset_album`, `filter_tracks` (small UI-state
  reset/search helpers that touch `FULL_TRACKS`).
- `album/tests.rs` (~30 lines) — the existing `#[cfg(test)] mod tests` (mmss,
  format_duration, tier unit tests), included via `#[cfg(test)] mod tests;`.

## Re-export surface
`album/mod.rs` is the public surface — since the crate presumably has
`mod album;` in `main.rs`/`lib.rs` and calls `crate::album::load_album(...)`
etc., keeping every currently-`pub` item re-exported (`pub use data::*;`,
`pub use map::load_album;`, `pub use apply::{apply_album, apply_artwork};`,
`pub use carousels::*;`, `pub use selection::*;`, `pub use reset::*;`) means no
caller elsewhere in the crate needs an import path change.

## Coupling / watch out
- `FULL_TRACKS` / `PLAY_TRACKS` thread-locals are read/written from `apply.rs`
  (write), `reset.rs` (clear), and `selection.rs` (read) — keep them declared
  once in `mod.rs` and reference via `super::FULL_TRACKS` / `super::PLAY_TRACKS`
  from the submodules; do not duplicate.
- `map.rs`'s `map_album`/`map_track` are called both by `load_album` (same file)
  and are the shared vocabulary `apply.rs` consumes (`AlbumData`/`TrackData`) —
  keep `data.rs` structs importable by both.
- Heavy cross-crate calls into `crate::artwork`, `crate::home`,
  `crate::album_map`, `crate::external_reco`, `crate::artist_blacklist`,
  `crate::fav_cache`, `crate::offline_cache`, `crate::custom_artwork`,
  `crate::booklet`, `crate::pinned`, `crate::immersive`, `crate::selection`,
  `crate::quality`, `crate::strip_html`, `crate::dates` are spread across nearly
  every proposed file — no special handling needed (crate-relative paths still
  resolve), just don't forget the `use` lines when splitting.
- `apply_album` is the single largest function (~200 lines) and is tightly
  coupled to `AlbumState`/`TrackItem`/`ArtistCredit`/`DiscoverSection` Slint
  types generated from `.slint` — do not attempt to split it further internally,
  it's already one cohesive "apply" operation.

## Verify after split
- `cargo check -p qbz` (or whichever crate name — check `Cargo.toml`) and
  `cargo build`.
- `cargo test -p qbz album` — the 3 existing unit tests (`mmss_pads_seconds`,
  `duration_drops_zero_hours`, `tier_classifies_bit_depth`) must stay green.
- Smoke-test: open an album view in the running app (via the `run` skill/manual
  launch) and confirm tracks, header, "more from artist", and "listening
  suggestions" carousels still render, and multi-select/track-search still work.
