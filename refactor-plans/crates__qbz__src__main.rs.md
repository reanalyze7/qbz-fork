# crates/qbz/src/main.rs (19,555 lines)

## Method note
This file is too large to read end-to-end line-by-line in one pass. This plan was
built by sampling: (1) `\grep`-ing every top-level `fn/struct/enum/impl/mod` signature
(234 matches) to map the ~7,250-line pre-`main()` section, (2) `\grep`-ing every
`.on_<name>(` Slint callback registration (370 distinct names) inside `fn main()`
itself, and (3) `\grep`-ing every distinct `global::<XyzState/XyzActions>` Slint
global binding with its FIRST line of appearance (125 distinct globals) to get an
ordered map of `main()`'s ~12,150-line body, cross-checked against inline comment
banners at several depths. Slint globals are declared per-feature-area in the `.slint`
UI, so their first-appearance order is a reliable proxy for `main()`'s internal
section boundaries even without reading every line in between.

## Summary
`crates/qbz/src/main.rs` is the Tauri-successor Slint desktop app's entry point. It
declares ~120 already-split business-logic submodules (`mod about; mod album; ...`,
lines 19-137), then ~7,250 lines of free functions (shell bootstrap, per-row
favorite/pin toggles, `navigate_*` page-transition helpers, a GPU/renderer-tier
detection subsystem, three existing `wire_playlist_manager`/`wire_myqbz`/
`wire_myqbz_detail` mega-functions), then one ~12,150-line `fn main()` that creates
the `AppWindow`, seeds every Slint global's initial state, and registers ~370
`.on_<callback>()` closures wiring UI actions to the business-logic submodules.

## Proposed split (24 top-level modules under `src/main/`)
Convert `crates/qbz/src/main.rs` into `crates/qbz/src/main/mod.rs` plus siblings.
Each numbered item below is a candidate file/module — most are still well over 130
lines and are explicitly flagged **[NEEDS 2ND PASS]** for a follow-up finer split
once someone does the full line-by-line read; this pass only identifies the
responsibility boundaries.

1. **`main/mod.rs`** (~150 lines) — the existing ~120 `mod X;` declarations (unchanged,
   lines 19-137), `dispatch(AppCommand)` (line 155), and a slimmed `fn main()` that
   just calls the `wire_*` functions below in sequence + keeps the final `Ok(())`.
2. **`main/shell_bootstrap.rs`** (~740 lines) `[NEEDS 2ND PASS]` — `init_shell_for_user`,
   `spawn_settings_snapshot_load`, `seed_tray_appearance`, `seed_blacklist_status`,
   `enter_shell`, `enter_shell_offline`, `current_genre_filter`, `reload_home`
   (orig. lines 155-892).
3. **`main/nav_flags_and_chrome.rs`** (~645 lines) `[NEEDS 2ND PASS]` —
   `current_browse_target`, `current_playlist_browse_showing`, `update_nav_flags`,
   `apply_resolved_link`, `select_all_active_surface`, `install_browser_mouse_nav`,
   `wire_window_controls`, `is_offline_blocked_view`, `safe_view_key`
   (orig. lines 892-1535).
4. **`main/row_toggles.rs`** (~835 lines) `[NEEDS 2ND PASS]` — every per-row
   favorite/pin/follow/cache-status mutation helper: `set_row_favorite`,
   `set_album_row_favorite`, `set_album_row_pinned`, `set_playlist_row_pinned`,
   `set_artist_row_pinned`, `set_pinned_album_favorite`, `set_playlist_row_following`,
   `record_search_interaction`, `playlist_copy_by_id`, `playlist_set_follow_by_id`,
   `playlist_toggle_favorite_by_id`, `toggle_track_favorite`, `myqbz_add_row_name`,
   `open_add_to_mixtape`, `mixtape_items_from_qobuz_tracks`,
   `mixtape_items_from_artist_selection`, `set_row_cache_status`,
   `set_row_unlocking`, `ensure_for_you_loaded` (orig. lines 1535-2369).
5. **`main/navigate_album_artist.rs`** (~430 lines) `[NEEDS 2ND PASS]` —
   `navigate_album`, `navigate_local_album`, `is_local_album_key`,
   `reveal_in_file_manager`, `navigate_artist` (orig. lines 2369-2800; `navigate_artist`
   alone is ~264 lines and is the prime candidate for further splitting).
6. **`main/playlist_picker_helpers.rs`** (~145 lines) — `picker_playlist_name`,
   `toast_added_tracks`, `toast_removed_tracks`, `toggle_off_playlist_pick`
   (orig. lines 2800-2944).
7. **`main/navigate_search_location.rs`** (~590 lines) `[NEEDS 2ND PASS]` —
   `navigate_search`, `scope_for`, `arm_scroll_restore`, `apply_entry`,
   `navigate_location`, `navigate_label`, `navigate_label_releases`,
   `navigate_artist_releases`, `navigate_suggestions` (orig. lines 2944-3537).
8. **`main/navigate_recent_library.rs`** (~740 lines) `[NEEDS 2ND PASS]` —
   `navigate_recent_albums`, `most_played_item`, `apply_most_played_page`,
   `navigate_most_played_albums`, `filter_most_played`, `refresh_recent_rails`,
   `library_albums_sorted`, `apply_library_albums_sort`, `navigate_favorites`,
   `navigate_library_all`, `open_local_artist`, `local_row_goto`,
   `navigate_local_library`, `navigate_mix`, `navigate_playlist`,
   `playlist_remove_rows`, `snapshot_detail_open` (orig. lines 3537-4279 — the
   biggest single "navigate_*" cluster; split by page type on the 2nd pass).
9. **`main/drag_and_sidebar.rs`** (~270 lines) `[NEEDS 2ND PASS]` — `local_drag_track`,
   `local_picker_ref`, `row_drag_track`, `gather_drag_tracks`,
   `load_sidebar_playlists`, `reconcile_sidebar_after_rename`,
   `refresh_sidebar_covers`, `reseed_i18n_labels` (orig. lines 4279-4547).
10. **`main/folder_editor.rs`** (~75 lines) — `open_folder_editor`,
    `refresh_pm_covers`, `folder_editor_presets` (orig. lines 4547-4623).
11. **`main/wire_playlist_manager.rs`** (~560 lines) `[NEEDS 2ND PASS — already one
    giant function]` — the existing `wire_playlist_manager` (orig. lines 4623-5183);
    move verbatim first, split its *body* into named handler functions in a later PR.
12. **`main/wire_myqbz.rs`** (~1,060 lines) `[NEEDS 2ND PASS — two giant functions]` —
    the existing `wire_myqbz` (~370 lines) and `wire_myqbz_detail` (~700 lines), plus
    `navigate_musician`, `system_font_family` (orig. lines 5183-6308).
13. **`renderer_select.rs`** (~1,100 lines) `[NEEDS 2ND PASS, but promote to a
    TOP-LEVEL crate module, not `main/` — this cluster is almost self-contained pure/
    detection logic]` — `RendererTier`, `RendererDecision`, `renderer_decision_summary`,
    all `renderer_sentinel_*`/`startup_probe_*`/`crash_chain_level` functions,
    `arm_auto_tier`, `GpuTopology`/`probe_gpu_topology`, `GpuAdapterInfo`/
    `gpu_adapters`, all `gpu_power_*` functions, `linux_has_system_battery`,
    `default_wgpu_power_preference`, `gpu_power_from_prefs`, `block_on_wgpu`,
    `create_shared_wgpu_stack`, `select_slint_backend`, `requested_renderer_tier`,
    `renderer_tier_from_prefs`, `detect_hardware_gpu`, `poll_ready`, `active_ui_scale`
    (orig. lines 6308-7409). On a 2nd pass this alone should become
    `renderer_select/{tier.rs, sentinel.rs, gpu_probe.rs, wgpu_setup.rs}`.
14. **`main/wire_startup_sequence.rs`** (~600 lines) `[NEEDS 2ND PASS]` — the
    imperative one-time boot sequence at the top of `fn main()`: UI-scale preset,
    logging setup, deep-link capture, artwork target sizing, single-instance guard,
    crash-chain watchdog arming, renderer selection invocation, `AppWindow::new()`,
    language resolution, sentinel disarm wiring, DPR persistence, shader underlay,
    dynamic-background setup, interface-scale preset, font test, window
    chrome/geometry restore, UI-prefs load, appearance/theme/tray seeding
    (orig. lines ~7409-7920). This is sequencing code, not callback registration —
    keep it that way rather than forcing it into the `wire_*` closure pattern.
15. **`main/wire_offline_and_auth.rs`** (~480 lines) `[NEEDS 2ND PASS]` —
    MusicBrainz-cache/image-cache init, audio+playback settings stores, offline-mode
    engine init + online/offline edge reactions, startup session restore, sign-in via
    browser, cancel-login, offline full-session entry, D2 banner re-login recovery
    (orig. lines ~7889-8365).
16. **`main/wire_search.rs`** (~730 lines) `[NEEDS 2ND PASS — split search vs
    cortinilla]` — open-album/open-artist top wiring, submit search, live-search
    debounce, results-tab switching, load-more, filter changes, and the entire
    "cortinilla" (live search dropdown) dismiss/move-selection/search-all/view-more/
    row-click wiring (orig. lines ~8365-9186).
17. **`main/wire_link_and_import.rs`** (~700 lines) `[NEEDS 2ND PASS]` — link-resolver
    actions/state (deep-link / pasted-URL resolution), playlist-import kickoff,
    settings-export actions, offline-mode actions, offline-favorites actions, myqbz
    branding state, pinned actions (orig. lines 9186-9889, keyed by first appearance
    of `LinkResolverActions` through `PinnedActions`).
18. **`main/wire_home_library_playback.rs`** (~2,340 lines)
    `[NEEDS 2ND PASS — LARGEST REMAINING CLUSTER, must be split further before this
    is usable as one file]` — the stretch between `PinnedActions` (9889) and
    `QueueState` (12230) with no new Slint global appearing in between; from the
    `.on_*` callback name census this covers album/artist row actions, discover-browse
    row actions, folders (local-library tree) actions, and playback transport
    controls. Recommend this becomes its own subdirectory
    (`main/wire_home_library_playback/{album.rs, artist.rs, folders.rs,
    playback.rs}`) on the 2nd pass rather than one file.
19. **`main/wire_queue_and_cards.rs`** (~680 lines) `[NEEDS 2ND PASS]` — queue state,
    sleep-timer actions, report-issue actions, album/artist/artist-releases actions,
    network-sidebar actions (orig. lines 12230-12911).
20. **`main/wire_info_modals_suggestions.rs`** (~720 lines) `[NEEDS 2ND PASS]` —
    track-info/album-info modal actions+state, musician/label page actions,
    suggestions + playlist-suggestions actions/state (orig. lines 12911-13631).
21. **`main/wire_discover_offline_manager.rs`** (~970 lines) `[NEEDS 2ND PASS]` —
    blacklist actions, offline-manager actions, location-view actions, home actions,
    most-played-albums actions, discover actions, external-reco actions, genre-filter
    actions/state (orig. lines 13631-14603).
22. **`main/wire_local_library_settings.rs`** (~1,625 lines) `[NEEDS 2ND PASS — 2nd
    largest cluster]` — local-library actions, the Settings screen state, library-
    manage actions, scrobble (Last.fm/ListenBrainz) actions, tag-editor actions/state,
    local-album actions/state (orig. lines 14603-16229). Likely splits cleanly into
    `settings.rs` / `scrobble.rs` / `tag_editor.rs` / `local_library.rs` on the 2nd pass.
23. **`main/wire_playlist_browse_picker.rs`** (~815 lines) `[NEEDS 2ND PASS]` —
    ephemeral-play-choice actions/state, discover-browse actions, playlist-browse
    actions, playlist-picker actions, duplicate-confirm state/actions
    (orig. lines 16229-17042).
24. **`main/wire_playlist_crud_sidebar.rs`** (~850 lines) `[NEEDS 2ND PASS]` — drag
    actions/state, playlist actions, edit-playlist actions, sidebar actions,
    create-folder actions/state (orig. lines 17042-17891).
25. **`main/wire_create_playlist_dac_import.rs`** (~990 lines) `[NEEDS 2ND PASS]` —
    create-playlist actions/state, DAC-wizard actions/state, sandbox state,
    playlist-import actions, favorites actions (orig. lines 17891-18880).
26. **`main/wire_library_all_artwork_close.rs`** (~675 lines) `[NEEDS 2ND PASS]` —
    library-all bulk actions, artwork/cover-management actions, and the final
    window-close/quit sequence (`on_close_requested`, `on_open_tos`, `window.show()`,
    `run_event_loop_until_quit()`, the macOS traffic-light centering, the exit-time
    audio/PipeWire cleanup) (orig. lines 18880-19555). Keep the exit-sequencing
    comments verbatim — they document subtle ordering requirements (hide-to-tray vs.
    quit, PipeWire clock reset on every quit path).

## Re-export / public-API surface
`main/mod.rs` is the only file anything outside this module touches (`fn main()` is
the binary entry point — nothing else in the crate calls into `crates/qbz/src/main.rs`
by path, since it's the `[[bin]]` root, not a library target). Internal cross-file
calls (e.g. `navigate_artist` called from `wire_search.rs`, `wire_myqbz` calling
`open_add_to_mixtape`) need `pub(crate)` or `pub(super)` visibility on the moved
functions plus `use crate::main::<module>::<fn>;` imports — since this is a binary
crate's root, everything can stay `pub(crate)` (nothing needs true `pub`).

## Tricky coupling to watch out for
- **This is a Slint app**: nearly every function takes `&AppWindow` or
  `slint::Weak<AppWindow>` and reaches into `window.global::<XyzState>()` /
  `window.global::<XyzActions>()`. There is no ownership boundary between these
  clusters beyond "which Slint global(s) it touches" — expect MANY cross-cluster
  calls (e.g. `navigate_album` in cluster 5 is almost certainly called from the open-
  album wiring in cluster 16, and from row-click handlers in cluster 18/19). Do NOT
  assume clean separation; grep for each function name across the whole file before
  moving it, to catch every call site.
- **Shared closures over `weak = window.as_weak()`**: many callback closures capture
  a cloned `weak` handle and other `Arc`/`Rc` state (image caches, offline engine
  handle, MusicBrainz cache, playback/audio settings stores) established once in
  cluster 14/15's startup sequence. Splitting `main()`'s body into `wire_*(&window,
  &shared_state...)` functions requires threading all of that shared state through
  function parameters — this is the single biggest mechanical risk of the whole
  split, more than any per-cluster line count.
- **`wire_playlist_manager` / `wire_myqbz` / `wire_myqbz_detail`** (clusters 11-12)
  are already extracted as standalone functions but are each single functions of
  370-700+ lines — moving them verbatim to their own file satisfies "one file per
  responsibility" but does NOT yet satisfy the 130-line rule; they need a real
  second-pass split (e.g. one handler function per `.on_*` callback they register).
- **`renderer_select` (cluster 13)** is the one cluster that is genuinely close to
  pure/self-contained (GPU detection, wgpu setup, sentinel files) and could become a
  real top-level crate module (not nested under `main/`) reusable/testable in
  isolation — worth doing first since it has the best effort/risk ratio.
- **Cluster 18** (`wire_home_library_playback`, ~2,340 lines) is the largest
  unresolved cluster — whoever does the real split MUST re-read that specific line
  range (9889-12230) in full before subdividing; this plan's cluster boundary here is
  the least confident one (derived from "no new Slint global appears in this range",
  not from reading the code).
- Order-of-registration matters for a few callbacks (e.g. `on_close_requested` must
  be registered before `window.show()`; the renderer sentinel/probe arm-then-disarm
  pair has an explicit race-condition comment) — preserve the exact relative order of
  statements within `fn main()` when moving code into `wire_*` calls, even across
  file boundaries.

## What to verify after the real split
- `cargo build -p qbz` (this crate historically had among the longest compile times
  in the workspace — watch for a regression or improvement from the split, both are
  informative).
- `cargo test -p qbz` if any unit tests exist in this file today (a `\grep` for
  `#[cfg(test)]` in the original file found none — check nothing was silently
  dropped).
- Manual smoke-test: launch the app, sign in, open an album/artist/playlist, search,
  toggle a favorite/pin, create a playlist, drag a track to a folder, put the app in
  offline mode and back online, open Settings and toggle a setting, quit via window
  close and via tray Quit — this file wires literally every user-facing action, so a
  full manual pass through the app's primary flows is the only realistic verification.
- Confirm `slint-viewer`/the Slint build step still finds every `.on_*` callback and
  every `global::<...>()` binding referenced from the split files (a moved callback
  registration that's accidentally dropped compiles fine in Rust but silently leaves
  a UI button non-functional — cargo build success is NOT sufficient proof here).
