# crates/qbz/src/label.rs (1204 lines)

## Summary
Two views in one file, each explicitly banner-separated: (1)
`LabelReleasesView` — header + paginated album catalog with sort/filter/
group-by-artist, and (2) the much larger `LabelPageView` "landing page" —
header + popular tracks + releases/critics/playlists/artists/more-labels
carousels, multi-select, and a pile of Qobuz JSON `Value`-parsing helpers.

## Proposed split
Split along the file's own `// ===...===` banner into two top-level
modules, then further by concern within each:

- `label/mod.rs` (~20 lines) — shared `PAGE_SIZE`, `bl_snapshots`,
  `extract_label_image` (used by both halves + the favorites Labels tab per
  its doc comment), `pub use` of `releases` and `page` submodules.
- `label/releases/mod.rs` (~90 lines) — `LabelData` struct, `load_label`,
  `load_more_albums`.
- `label/releases/view.rs` (~140 lines) — `apply_label`, `derive_releases`
  (the sort/filter/group-by-artist re-derivation — the single biggest
  function in this half), `append_albums`, `apply_image`, `reset_label`,
  `artwork_jobs`. Still slightly over — split `derive_releases` (~75 lines)
  into its own `label/releases/derive.rs` if the reviewer wants each file
  strictly under 130.
- `label/page/mod.rs` (~70 lines) — `LabelPagePayload` + its five small
  slim structs (`TopTrack`, `PlaylistSlim`, `ArtistSlim`, `LabelSlim`),
  `PLAY_TOP_TRACKS` thread_local, `top_tracks_for_play`, `pub use` of
  submodules.
- `label/page/selection.rs` (~80 lines) — `set_multi_select`,
  `recount_selected`, `select_all`, `clear_selection`, `selected_ids`,
  `selected_play_tracks` (Popular Tracks multi-select — mirrors the pattern
  seen elsewhere, e.g. `selection.rs`).
- `label/page/load.rs` (~135 lines) — `load_label_page`,
  `favorite_label_ids`. Slightly over — could move `favorite_label_ids`
  (~20 lines) into `label/page/mod.rs` instead.
- `label/page/parse.rs` (~145 lines) — `parse_top_track`, `parse_playlist`,
  `parse_artist`, `parse_more_labels` (the Value → slim-struct mappers).
  Still over — split `parse_top_track` alone (~75 lines, the most complex
  one with the main-artist-role resolution logic) into
  `label/page/parse_track.rs`, leaving `parse_playlist`/`parse_artist`/
  `parse_more_labels` (~70 lines) in `parse.rs`.
- `label/page/value_helpers.rs` (~100 lines) — `value_to_string`,
  `name_display`, `parse_image_value`, `parse_artist_image`,
  `parse_playlist_image`, `parse_explore_image`, `mmss`, `truncate_words`
  (the small pure Value-extraction helpers, explicitly commented as
  "mirror the Svelte getX helpers").
- `label/page/to_slint.rs` (~100 lines) — `top_track_to_item`,
  `playlist_to_item`, `artist_to_item`, `label_to_item`, `section` (Slint
  item mapping).
- `label/page/apply.rs` (~130 lines) — `apply_label_page`,
  `apply_label_library`, `build_label_jump_tabs`, `page_artwork_jobs`.
  At budget — split `build_label_jump_tabs` (~40 lines) out if it grows.
- `label/page/follow.rs` (~50 lines) — `label_following_state`,
  `more_label_name`, `mark_label_followed`.
- `label/page/reset.rs` (~30 lines) — `reset_label_page`.

## Re-export surface
`label/mod.rs` stays the `mod label;` target. Public API used by
`main.rs`/other views (`load_label`, `apply_label`, `load_more_albums`,
`append_albums`, `apply_image`, `reset_label`, `artwork_jobs`,
`load_label_page`, `apply_label_page`, `apply_label_library`,
`reset_label_page`, `top_tracks_for_play`, `set_multi_select`,
`recount_selected`, `select_all`, `clear_selection`, `selected_ids`,
`selected_play_tracks`, `label_following_state`, `more_label_name`,
`mark_label_followed`, `page_artwork_jobs`) all re-exported via `pub use
releases::*; pub use page::*;` (with page's own `pub use` chain of its
submodules) so every `crate::label::X` call site is unchanged.
`pub(crate) fn extract_label_image` (used cross-module by favorites) stays
`pub(crate)` in `label/mod.rs`.

## Coupling / watch out
- `PLAY_TOP_TRACKS: thread_local!` caches the landing page's queueable
  tracks for "Play all"; it's set in `apply_label_page` and read by
  `top_tracks_for_play` — both must stay reachable from wherever a "play
  the label's top tracks" action lives (likely `main.rs`); keep the
  thread_local definition and its two accessors together in `page/mod.rs`.
- `extract_label_image`'s doc comment explicitly says it's "Reused by the
  favorites Labels tab, whose wire `image` is a bare string" — this is a
  real external caller outside this file; do not accidentally make it
  private during the split.
- `derive_releases`'s "fast path" (no filter/search/sort='newest'/no-group)
  reuses the LIVE `albums` model directly so artwork keeps updating in
  place — a comment calls this out explicitly as the reason artwork stays
  live in the common case; preserve this fast-path branch exactly, first,
  before the general filter/sort/group path, in whichever file houses it.
- `parse_top_track`'s main-artist resolution (roles array containing
  "main-artist", falling back through `performer`/`artist`/`album.artist`)
  is annotated "discussion #631" — a real bugfix with history; do not
  simplify this fallback chain during the split.
- `top_track_to_item`, `playlist_to_item` etc. call
  `crate::artist_blacklist::stamp_row`, `crate::pinned::is_pinned`,
  `crate::fav_cache::is_favorite`, `crate::offline_cache::is_cached` — keep
  those `crate::` imports wherever `to_slint.rs` lands.
- `bl_snapshots()` (mod.rs) is called by BOTH `load_label`/`load_more_albums`
  (releases half) AND (indirectly, via album_blacklisted filtering)
  `load_label_page`'s critics/releases-carousel builders — must stay
  reachable from both `releases/` and `page/` submodules via
  `use super::bl_snapshots;` (or `use crate::label::bl_snapshots;`).

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap;
  the Value-parsing helpers especially would benefit from unit tests in a
  real split PR).
- Smoke-test both views: LabelReleasesView (header + album grid + load-more
  + sort/filter/group-by-artist toggle), and the rich LabelPageView landing
  (popular tracks + all five carousels + multi-select bulk bar + follow
  toggle on header and more-labels cards).
