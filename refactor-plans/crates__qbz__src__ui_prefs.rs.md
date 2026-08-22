# crates/qbz/src/ui_prefs.rs (698 lines)

## 1. Summary

A JSON-backed UI-preference store: the `UiPrefs` struct (~60 fields
covering streaming quality, language, appearance, window geometry,
renderer/GPU choices, keybindings, etc.), its serde defaults, index<->key
mapping helper functions for every dropdown-backed setting, `load()`/
`save()` I/O, and unit tests.

## 2. Proposed module split

Convert `ui_prefs.rs` into `ui_prefs/` with a `mod.rs` barrel:

| New file | Owns | ~lines |
|---|---|---|
| `ui_prefs/mod.rs` | Module declarations + re-exports only | ~20 |
| `ui_prefs/quality.rs` | `StreamingQuality`, `STREAMING_QUALITIES`, `DEFAULT_STREAMING_QUALITY`, `streaming_quality_for_key`, `streaming_quality_index`, `default_streaming_quality` | ~65 |
| `ui_prefs/index_maps.rs` | All the other index<->key mapping pure functions: `language_for_index`/`language_index`, `auto_theme_source_for_index`/`_index`, `startup_page_for_index`/`_index`, `renderer_for_index`/`_index`, `ui_scale_for_index`/`_index`/`_factor`, `app_background_for_index`/`_index`, `large_spectrum_mode_index`/`_key` | ~130 |
| `ui_prefs/defaults.rs` | The small `default_*()` free functions backing serde `#[serde(default = "...")]` attributes (`default_system_notifications`, `default_musicbrainz_enabled`, `default_nav_in_sidebar`, `default_volume`, `default_startup_page`, `default_last_view`, `default_window_pos`, `default_use_system_title_bar`, `default_show_window_controls`, `default_wc_position`, `default_gpu_power`, `default_renderer`, `default_ui_scale`, `default_last_dpr`, `default_language`, `default_large_visualizer`, `default_large_spectrum_mode`, `default_album_header_gradient`, `default_intelligent_search`, `default_window_title_show`, `default_show_volume_steppers`, `default_sidebar_playlist_collage`, `default_local_library_track_artwork`, `default_in_app_toasts`, `default_theme_filter`, `default_app_background`, `default_theme`, `default_auto_theme_source`) | ~110 |
| `ui_prefs/model.rs` | The `UiPrefs` struct definition (all `#[serde(default=...)]` fields + doc comments) and its `impl Default for UiPrefs` | ~150 |
| `ui_prefs/io.rs` | `prefs_path()`, `load()`, `save()` — the actual filesystem I/O | ~45 |
| `ui_prefs/tests.rs` | The `#[cfg(test)] mod tests` block, `use super::*` swapped for explicit imports from the sibling modules | ~50 |

This follows the pure/IO split cleanly: `model.rs` + `quality.rs` +
`index_maps.rs` + `defaults.rs` are pure data/logic, `io.rs` is the only
file touching `std::fs`/`dirs`.

## 3. Re-export / public API surface

`ui_prefs/mod.rs` re-exports everything the rest of the crate currently
reaches via `qbz::ui_prefs::X`:

```rust
mod defaults;
mod index_maps;
mod io;
mod model;
mod quality;
#[cfg(test)]
mod tests;

pub use index_maps::*;
pub use io::{load, save};
pub use model::UiPrefs;
pub use quality::{
    StreamingQuality, DEFAULT_STREAMING_QUALITY, STREAMING_QUALITIES,
    streaming_quality_for_key, streaming_quality_index,
};
pub use model::DEFAULT_ALBUM_HEADER_GRADIENT... // (constants currently pub in the file — keep all pub items re-exported)
```

Callers today do `use qbz::ui_prefs::{UiPrefs, load, save, ...};` — this
keeps every one of those paths working unchanged since `ui_prefs` becomes
a directory module with the same public surface.

## 4. Tricky coupling to watch out for

- The `default_*()` free functions in `defaults.rs` are referenced *by
  name* from `#[serde(default = "default_xxx")]` attributes inside the
  `UiPrefs` struct in `model.rs` — these must stay resolvable via
  `use crate::ui_prefs::defaults::*;` (or fully qualified) inside
  `model.rs`, and serde's `default = "path"` attribute needs the fully
  resolvable path string, not just an in-scope name, if it's not directly
  in the same module — confirm serde accepts `default =
  "crate::ui_prefs::defaults::default_volume"` or keep a `use` glob and
  short names for minimal edit risk.
- `impl Default for UiPrefs` in `model.rs` also calls every `default_*()`
  helper directly (not just via serde) — same cross-module reference.
- `qbz_theme::default_slug()` is called from `default_theme()` — verify
  the `qbz_theme` crate dependency is available from whichever file ends
  up owning `default_theme()` (goes in `defaults.rs`).
- Tests currently do `use super::*` to reach everything in one flat
  file; after the split they need explicit `use
  crate::ui_prefs::{...}` imports across module boundaries — enumerate
  every symbol each test function touches (`UiPrefs`, `STREAMING_QUALITIES`,
  `streaming_quality_index`, `streaming_quality_for_key`) since they now
  live in different files.

## 5. What to verify after the real split

- `cargo build -p qbz` and `cargo test -p qbz ui_prefs::` — all 6
  existing tests (`default_is_hires_plus`, `unknown_key_resolves_to_default_index`,
  `quality_key_maps_to_qobuz_format_id`, `legacy_json_without_field_deserializes`,
  `default_theme_is_oled`, plus the crate-wide serde round-trip) stay
  green.
- `cargo build` for every crate that imports `qbz::ui_prefs::*` (grep for
  `ui_prefs::` usages, e.g. in `qbz-app`/`qbzd`/`qbz-ui` controllers) to
  confirm no path broke.
- Smoke-test app startup: `ui_prefs.json` load/save round-trip still
  works (delete a test profile's file, launch, confirm defaults, change a
  setting, restart, confirm persisted).
