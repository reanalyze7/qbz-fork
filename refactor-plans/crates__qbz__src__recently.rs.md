# crates/qbz/src/recently.rs (281 lines)

## Summary
Recently-played store: a small JSON file holding the last-played tracks and a
separately-capped list of recently-played albums (both newest-first,
deduplicated by id), read by Discover Home's two "Recently Played" rails and
written by the playback session via `record`.

## Proposed split

- `recently/mod.rs` (~25 lines) — module doc, `mod` declarations, `pub use`
  re-exports of every public item.
- `recently/album_meta.rs` (~35 lines) — `AlbumMeta` struct, the `ALBUM_META`
  static, `remember_album_meta`, `album_meta` — the album-metadata cache fed
  by the playback album-fetch paths, logically separate from the
  track/album history store itself.
- `recently/model.rs` (~85 lines) — `RecentTrack`, `RecentAlbum`,
  `RecentStore` structs (the persisted data shapes).
- `recently/store_io.rs` (~90 lines) — `store_path`, `derive_albums`,
  `read_store`, `write_store` — the file-I/O + legacy-migration logic.
- `recently/api.rs` (~75 lines) — `load`, `load_albums`, `prune_albums`,
  `record` — the public read/write API called by Discover Home and the
  playback session.

## Re-export surface
`recently/mod.rs` re-exports `AlbumMeta`, `RecentTrack`, `RecentAlbum`,
`remember_album_meta`, `album_meta`, `load`, `load_albums`, `prune_albums`,
`record` at `crate::recently::*` — Discover Home's rail-loading code and the
playback session's `record` call keep working unchanged.

## Tricky coupling / watch out
- `RecentStore` (in `model.rs`) is a private struct only ever constructed/read
  in `store_io.rs` — keep it `pub(crate)` or module-private to `recently/`
  rather than fully `pub`, matching its current (implicit, file-private)
  visibility.
- `derive_albums` (in `store_io.rs`) is the ONE place that recreates the
  pre-#567 legacy behavior (deriving albums from the track window) — it's
  used only inside `read_store`'s fallback branch. Keep the doc comment
  explaining this is a one-time migration path, not a general-purpose
  derivation — a future refactor might otherwise be tempted to call it
  elsewhere.
- `MAX_RECENT` and `MAX_RECENT_ALBUMS` are independent caps (the #567 fix —
  album history no longer derives from the 24-track window) — both
  constants are referenced from `api.rs`'s `record` function; keep them
  defined once (e.g. in `model.rs` next to the structs they bound, or in
  `store_io.rs`) and imported, not redefined.
- `record`'s two dedup-and-truncate blocks (tracks vs. albums) are
  independent but both live in one function today — if `api.rs` stays under
  130 lines with them combined, no further split is needed; if not, they can
  become `record_track`/`record_album` private helpers called by `record`.

## What to verify after the real split
- `cargo build -p qbz` (no `#[cfg(test)]` block exists in this file today;
  confirm no test regressions crate-wide).
- Grep for `crate::recently::` in `crates/qbz/src/` (expected in the Discover
  Home rail-building code and wherever the playback session records a
  played track) to confirm call sites are unaffected.
- Manual smoke test via the `run` skill: play a track, confirm it appears in
  "Recently Played" and its album in "Recently Played Albums"; delete a
  Local Library folder and confirm `prune_albums` still removes its
  entries from both rails.
