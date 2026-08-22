# crates/qbz/src/home.rs (1168 lines)

Discover/Home controller: fetch discover index -> plain data (worker
thread) -> Slint models on HomeState/DiscoverState, plus the prefs-driven
descriptor rendering (Slice 5) and Qobuz-Playlists tag filter.

## Proposed split

- `home/mod.rs` (~120 lines) — re-export surface + all public data types
  (`HomeData`, `SectionData`, `CardData`, `PlaylistCardData`, `SlimData`)
  and the `TAB_SECTIONS` thread_local + `TabSections` struct +
  `filter_playlists`. Everything else imports from here.
- `home/load.rs` (~200 lines) — `load_home` (the big async fetch +
  blacklist-filter + section-push pipeline), `recent_track_slims`,
  `recent_album_cards`.
- `home/map.rs` (~230 lines) — the pure Discover -> Card/Slim/Playlist
  mappers: `push_section`, `push_section_ref`, `map_album`, `map_playlist`,
  `classify_release_type`, `quality_detail`, `map_slim`, `quality_tier`,
  `quality_label`, `format_rate`.
- `home/present.rs` (~230 lines) — Slint conversion + state push:
  `slim_to_item`, `apply_recent_rails`, `card_to_item`, `playlist_to_item`,
  `build_sections`, `playlist_artwork_jobs`, `apply_home`.
- `home/descriptors.rs` (~250 lines) — the Slice-5 prefs-driven descriptor
  system: `HOME_RENDERABLE`, `descriptor_section`, `descriptors_for`,
  `tab_descriptors`, `discover_section_artwork_jobs`, `rerender_active_tab`,
  `select_tab`.
- `home/tags.rs` (~90 lines) — the Qobuz-Playlists category-tag filter:
  `rerender_playlists_filtered`, `toggle_playlist_tag`,
  `clear_playlist_tags`, `sync_tag_selection`.
- `home/tests.rs` (~50 lines) — existing `#[cfg(test)]` block.

## Tricky coupling — the big one in this slice

- `TAB_SECTIONS` thread_local is read/written from `load.rs` (via
  `apply_home`... actually `present.rs`), `descriptors.rs`, and `tags.rs`.
  Keep it `pub(super)` in `mod.rs` so every submodule can
  `crate::home::TAB_SECTIONS.with(...)`.
- `select_tab` (descriptors.rs) calls `rerender_active_tab` (same file) and
  reaches into `crate::discover_prefs` — no change needed.
- Heavy cross-file consumers: `main.rs`/other Slint controllers likely call
  `home::apply_home`, `home::load_home`, `home::select_tab`,
  `home::toggle_playlist_tag` etc. directly — re-exporting all pub fns
  from `mod.rs` keeps `crate::home::X` paths stable.

## Verify after split

`cargo build -p qbz`, `cargo test -p qbz home::`, manual: Home tab loads
sections, Editor's Picks tab switch, Qobuz Playlists tag filter, Library
Albums/Release Watch/Top Artists rails render.
