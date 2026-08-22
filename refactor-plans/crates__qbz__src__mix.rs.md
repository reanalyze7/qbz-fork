# crates/qbz/src/mix.rs (515 lines)

## Summary
Qobuz mix detail views (DailyQ/WeeklyQ/FavQ/TopQ): resolves each mix kind
into a track list (dynamic/suggest seeded from history+favorites, favorite
shuffle, or playlist aggregation), maps to `TrackItem`s, applies to
`MixState`, and provides its own multi-select block (mirrors `label.rs`'s
pattern).

## Proposed split
By concern — mix resolution (per-kind data fetching) vs Slint mapping/state
vs multi-select:

- `mix/mod.rs` (~50 lines) — `CURRENT_MIX` static, `mix_meta`, `pick_spread`,
  `pub use` of submodules.
- `mix/resolve.rs` (~140 lines) — `mix_listened_seed_ids`,
  `build_tracks_to_analyse`, `load_mix` (the DailyQ/WeeklyQ/FavQ/TopQ
  resolution). Still over — split the DailyQ/WeeklyQ branch (the
  seed+analyse+dynamic-suggest chain, ~90 lines including
  `mix_listened_seed_ids`+`build_tracks_to_analyse`) into
  `mix/resolve/daily_weekly.rs`, leaving `load_mix`'s dispatch + the
  fav/top branches (~50 lines) in `resolve.rs`.
- `mix/sources.rs` (~35 lines) — `favorite_tracks`, `playlist_tracks` (the
  two simple data-source fetchers used by fav/top).
- `mix/shuffle.rs` (~20 lines) — `shuffle` (the deterministic xorshift
  shuffle, shared by fav-mix loading and the Shuffle action).
- `mix/to_slint.rs` (~90 lines) — `mmss`, `to_item`, `total_duration` (Track
  → TrackItem/display mapping).
- `mix/state.rs` (~35 lines) — `apply_mix`, `reset_mix`.
- `mix/queue_access.rs` (~20 lines) — `current_tracks`, `shuffled_tracks`,
  `index_of` (reads of the cached `CURRENT_MIX`).
- `mix/selection.rs` (~85 lines) — `set_multi_select`, `recount_selected`,
  `select_all`, `clear_selection`, `selected_ids`, `selected_play_tracks`
  (the multi-select block — near-identical to `label.rs`'s and
  `myqbz_detail`-style patterns elsewhere).
- `mix/artwork.rs` (~15 lines) — `artwork_jobs`.

## Re-export surface
`mix/mod.rs` stays the `mod mix;` target. Public API used by `main.rs`
(`mix_meta`, `load_mix`, `apply_mix`, `reset_mix`, `current_tracks`,
`shuffled_tracks`, `index_of`, `artwork_jobs`, `set_multi_select`,
`recount_selected`, `select_all`, `clear_selection`, `selected_ids`,
`selected_play_tracks`) re-exported via `pub use resolve::*; pub use
state::*; pub use queue_access::*; pub use selection::*; pub use
artwork::*;` so `crate::mix::X` call sites are unchanged.

## Coupling / watch out
- `CURRENT_MIX: LazyLock<Mutex<Vec<Track>>>` is shared across `state.rs`
  (`apply_mix` writes it), `queue_access.rs` (`current_tracks`/
  `shuffled_tracks` read it), and `selection.rs`'s
  `selected_play_tracks` (reads via `current_tracks()`) — must stay
  defined in `mod.rs`, all consumers `use super::CURRENT_MIX;` or just call
  the public `current_tracks()` accessor where possible (cleaner).
- `shuffle()` (mix/shuffle.rs) is called from BOTH `resolve.rs`'s "fav"
  branch AND `queue_access.rs`'s `shuffled_tracks` — keep it a free
  `pub(super)` fn so both can `use super::shuffle::shuffle;`.
- The DailyQ/WeeklyQ resolution comment explicitly documents a two-tier
  fallback (reco-backed scored seeds -> local recents+favorites derivation)
  and a further fallback within `load_mix` itself (empty analysed result ->
  retry with empty analysis) — preserve BOTH fallback layers exactly; this
  is exactly the kind of nested fallback logic that's easy to accidentally
  collapse during a "simplifying" split.
- `to_item` (to_slint.rs) computes the blacklist stamp from performer OR
  composer id, explicitly commented "Task 6"/"D-FEAT" — matches the queue's
  blacklist predicate; don't diverge the two independently.
- This file duplicates the multi-select block pattern seen in `label.rs`
  nearly verbatim (`set_multi_select`/`recount_selected`/`select_all`/
  `clear_selection`/`selected_ids`/`selected_play_tracks`) — flag as a
  cross-file extraction opportunity (a generic multi-select-over-model
  helper) for a follow-up, not something to fix silently here.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap;
  `pick_spread`, `shuffle`, and `to_item`'s blacklist-key derivation are
  good unit-test candidates for a real split PR).
- Smoke-test all four mix kinds (Daily/Weekly/Fav/Top), the Shuffle action,
  and multi-select bulk actions on the mix track list.
