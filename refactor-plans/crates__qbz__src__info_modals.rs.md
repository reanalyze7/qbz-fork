# crates/qbz/src/info_modals.rs (563 lines)

## Summary
Track Info + Album Info (Credits/Review) modal controllers: fetch fresh data
through `QbzCore` (`get_track`/`get_album`), map it to plain `Send` structs
on the worker thread, then apply it to the `TrackInfoState`/`AlbumInfoState`
Slint globals on the event loop (1:1 port of Tauri's `TrackInfoModal.svelte`
+ `AlbumCreditsModal.svelte`).

## Proposed split
By pipeline stage — a `info_modals/` module directory with a thin `mod.rs`
re-export.

- `info_modals/mod.rs` (~35 lines) — module declarations +
  `pub use spawn::{open_track_info, load_track_info_inline, open_album_credits};`
  so `crate::info_modals::open_track_info` etc. keep their current paths.
- `info_modals/types.rs` (~60 lines) — the plain worker-thread data structs:
  `CreditRowData`, `TrackInfoData`, `PerformerData`, `AlbumTrackData`,
  `AlbumCreditsData`. Give every field `pub(super)` (currently private,
  fine within one file, but needed across files once `map.rs`/`apply.rs`
  construct/read them).
- `info_modals/format.rs` (~95 lines) — the pure formatting helpers:
  `format_title`, `track_duration`, `album_duration`, `fmt_rate`,
  `track_quality`, `album_quality`, `full_release_date`, `roles_suffix`.
  Zero dependency on Slint or the data structs — easy to unit test in
  isolation later.
- `info_modals/map.rs` (~165 lines) — `map_track_info`, `map_album_credits`
  (the `Track`/`Album` → plain-struct mapping, using `types` + `format` +
  `qbz_qobuz::performers`).
- `info_modals/apply.rs` (~105 lines) — `apply_track_info`,
  `apply_album_credits` (writes into `TrackInfoState`/`AlbumInfoState` on
  the Slint event loop; builds the paired-column credit model).
- `info_modals/spawn.rs` (~115 lines) — the public entry points:
  `spawn_track_info` (shared fetch+map+apply helper), `open_track_info`,
  `load_track_info_inline`, `open_album_credits`. Depends on `map` + `apply`.

## Re-export surface
`info_modals/mod.rs` is the public-API surface: `pub use spawn::*` (or named
re-exports of the 3 public fns) so every caller (the media-action handler)
keeps using `crate::info_modals::open_track_info(...)` etc. unchanged.

## Coupling / watch out
- `TrackInfoData`/`AlbumCreditsData`/`CreditRowData`/`PerformerData`/
  `AlbumTrackData` fields are currently private (no `pub` at all, since
  everything lived in one file/module). Moving them to `types.rs` and
  constructing/reading them from `map.rs`/`apply.rs` requires bumping field
  visibility to at least `pub(super)` (or `pub(crate)` if simpler) — a
  mechanical but easy-to-miss step; the compiler will catch every missed
  field immediately.
- `map_track_info`/`map_album_credits` and `apply_track_info`/
  `apply_album_credits` both import the same generated Slint types
  (`AlbumCreditPerformer`, `AlbumCreditTrack`, `AlbumInfoState`, `AlbumState`,
  `AppWindow`, `InfoCreditPair`, `InfoCreditRow`, `TrackInfoState`) from
  `crate::{...}` — keep those `use` lines in `apply.rs` only (that's the
  only file touching Slint state); `map.rs` should only need
  `qbz_models::{Album, Track}` plus its own `types`.
  `crate::strip_html` and `crate::dates::current_locale` are used inside
  `map.rs` (album/track mapping), not `apply.rs` — verify imports land in
  the right file.
- `spawn_track_info`'s `open_modal: bool` parameter distinguishes the
  explicit (i)-button flow from the immersive-panel inline-load flow — this
  nuance lives entirely in `spawn.rs`; don't lose the doc comment when
  splitting.
- Generic bound `A: FrontendAdapter + Send + Sync + 'static` repeats on
  every public spawn function — keep it identical across `spawn.rs`'s
  functions (no behavior change intended by the split).

## Verify after split
- `cargo build -p qbz`
- `cargo test -p qbz` (no existing unit tests in this file today, but
  confirm the crate still builds and any adjacent tests referencing these
  types still pass).
- Grep for `info_modals::` usage across `qbz` (media-action handler, main
  window setup) to confirm no import path broke.
- Manual smoke test: open Track Info modal via the (i) button, open the
  immersive Track Info panel (inline load, no modal popup), open Album Info
  (Credits/Review) modal — confirm data renders identically to before the
  split (title, duration, quality string formatting, credits grouping,
  album review text, per-track credits/copyright).
