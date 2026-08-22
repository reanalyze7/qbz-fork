# crates/qbz-offline-cache/src/types.rs (147 lines)

## Summary
Pure serde DTOs shared by the offline cache subsystem (moved verbatim from
the old Tauri `offline_cache/mod.rs`): `OfflineCacheStatus` enum (+ its
str conversions and cover-path resolution logic on `CachedTrackInfo`),
`ReadyTrackForSync`, `OfflineCacheStats`, `CacheProgress`, `TrackCacheInfo`.

## Proposed split
Barely over budget (147 lines) — split status/track-info (the two types
with real logic) from the three plain data-only structs:

- `types/mod.rs` (~70 lines) — module doc, `OfflineCacheStatus` enum + its
  `as_str`/`from_str` impl (lines 6-35), `pub use` re-exports of the other
  submodule's items.
- `types/track_info.rs` (~65 lines) — lines 37-109: `CachedTrackInfo` struct
  + its `resolve_cover_path` impl (the one piece of real logic in this
  file — the 3-tier cover-thumbnail fallback), and `ReadyTrackForSync`
  (small, grouped here since both represent "a track's cache info", though
  it could equally sit in mod.rs — either placement is fine, just keep it
  out of mod.rs if mod.rs is otherwise purely the status enum).
- `types/stats.rs` (~40 lines) — lines 111-147: `OfflineCacheStats`,
  `CacheProgress`, `TrackCacheInfo` — the three plain data-only structs with
  no associated logic.

Given the file is only 17 lines over, an equally valid minimal-diff option
is a two-way split (`mod.rs` = status enum + the 3 plain structs, `track_
info.rs` = `CachedTrackInfo` + `resolve_cover_path` + `ReadyTrackForSync`) —
either grouping keeps every file well under 130 lines; pick whichever reads
more naturally when actually doing the split.

## Re-export surface
`types/mod.rs` becomes the `mod types;` target already referenced as
`qbz_offline_cache::types::X` from wherever the offline-cache crate's
`lib.rs`/`mod.rs` re-exports it (and from `qbz`'s usages like
`qbz_offline_cache::OfflineCacheStatus`, `CachedTrackInfo`, etc. — check
whether the crate root already does `pub use types::*;`, in which case
nothing downstream changes at all). Every struct/enum here must stay
reachable at its current path via `pub use track_info::*; pub use stats::*;`
in `types/mod.rs`.

## Coupling / watch out
- `CachedTrackInfo::resolve_cover_path` is called from at least
  `crates/qbz/src/local_playlist.rs` (seen in this same batch — the
  offline-cache-resolved `RowItem::Cached` construction) and likely other
  offline-mode read paths — a purely mechanical move, no logic changes
  needed, but worth flagging: this is the SAME `resolve_cover_path` method
  referenced by files another agent may be splitting in `qbz/src/`, so the
  import path `qbz_offline_cache::CachedTrackInfo` (or wherever it's
  re-exported from) must not change.
- `OfflineCacheStatus::from_str` has a fail-open default (`_ =>
  Self::Failed`) — preserve this exactly, it's a deliberate "unknown status
  string renders as Failed" contract, not an oversight.
- No `#[cfg(test)]` in this file — verification is compile + smoke-test only.

## Verify after split
- `cargo check -p qbz-offline-cache` and `cargo build -p qbz-offline-cache`.
- `cargo build -p qbz` (or the full workspace) to confirm every downstream
  crate importing `OfflineCacheStatus`/`CachedTrackInfo`/etc. still resolves.
- No dedicated runtime smoke-test needed beyond the normal offline-cache
  flows (download a track, verify its cover resolves) since this is a pure
  data/serde file with no I/O of its own beyond the one cover-path
  filesystem check.
