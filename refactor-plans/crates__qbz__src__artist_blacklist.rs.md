# crates/qbz/src/artist_blacklist.rs (367 lines)

## Summary
Per-user artist/album-blacklist lifecycle + thin access wrapper around
`qbz_app::settings::artist_blacklist::BlacklistService` (a process-global
`Mutex<Option<Service>>` bound per session), providing fail-open read
accessors, the shared row/queue "should hide this" predicates
(`stamp_row`/`is_track_blacklisted`), and mutation fns.

## Proposed split
Clean sequential file already marked by `// ---- ---- ----` section
comments — split along those.

- `artist_blacklist/mod.rs` (~35 lines) — module doc, `pub mod` decls,
  `pub use` re-exports, the `SERVICE` static + `NO_SESSION_ERR` const (shared
  process-global state every submodule touches).
- `artist_blacklist/lifecycle.rs` (~20 lines) — `init_for_user`, `teardown`.
- `artist_blacklist/reads.rs` (~120 lines) — `with_service` helper,
  `is_blacklisted`, `is_blacklisted_id_str`, `is_album_blacklisted`,
  `card_blacklisted`, `is_enabled`, `ids_snapshot`, `get_all`, `count`,
  `album_ids_snapshot`, `get_all_albums`, `album_count`.
- `artist_blacklist/predicates.rs` (~50 lines) — `stamp_row`,
  `is_track_blacklisted` (the two heavily-documented shared row/queue
  predicates — kept together since their doc comments cross-reference each
  other and the invariant "greys out == drops from queue" is easiest to
  audit in one place).
- `artist_blacklist/mutations.rs` (~65 lines) — `mutate` helper, `add`,
  `remove`, `set_enabled`, `clear_all`, `add_album`, `remove_album`,
  `clear_all_albums`.
- `artist_blacklist/tests.rs` (~75 lines) — the `#[cfg(test)] mod tests`
  block (`unique_temp_dir` + the single combined `lifecycle_roundtrip` test).

## Re-export surface
`artist_blacklist/mod.rs` re-exports every public fn
(`init_for_user`/`teardown`/`is_blacklisted`/`is_blacklisted_id_str`/
`is_album_blacklisted`/`card_blacklisted`/`stamp_row`/
`is_track_blacklisted`/`is_enabled`/`ids_snapshot`/`get_all`/`count`/
`album_ids_snapshot`/`get_all_albums`/`album_count`/`add`/`remove`/
`set_enabled`/`clear_all`/`add_album`/`remove_album`/`clear_all_albums`) at
`crate::artist_blacklist::*` so every caller across the `qbz` frontend crate
(`search.rs` calls `crate::artist_blacklist::is_enabled`/`ids_snapshot`/
`album_ids_snapshot` directly, per the search.rs read above) keeps working
unchanged.

## Coupling / watch out
- `SERVICE: Mutex<Option<BlacklistService>>` is a process-global `static` —
  it must be defined exactly once; keep it in `mod.rs` and have every other
  submodule reference `super::SERVICE` (or `crate::artist_blacklist::SERVICE`
  if made `pub(crate)`).
- `stamp_row` and `is_track_blacklisted` are DELIBERATELY coupled — the
  doc comments explicitly say `is_track_blacklisted` "funnels" through
  `stamp_row` so render greyout and queue-drop can never diverge; keep them
  in the same file (as proposed) so this invariant stays easy to review, and
  never let a future edit reimplement one independently of the other.
- `search.rs` (a sibling file also in this batch) calls
  `crate::artist_blacklist::is_enabled()`, `ids_snapshot()`, and
  `album_ids_snapshot()` in multiple places (`load_search`, `load_cortinilla`,
  `load_immersive_search`, `load_more`) — these three fn signatures must not
  change shape during the split.
- `with_service`/`mutate` are the two "fail-open vs fail-with-error" access
  patterns — every read fn goes through `with_service`, every mutation goes
  through `mutate`; keep both helpers visible to their respective submodules.

## Verify after split
- `cargo test -p qbz artist_blacklist::tests::lifecycle_roundtrip` — the one
  combined test must stay green (exercises init/add/snapshot/album axis/
  stamp_row/teardown end-to-end; note it uses the process-global `SERVICE`,
  so it cannot safely run concurrently with other tests touching the same
  static — check if `#[test]` serialization already handles this before and
  after the split).
- `cargo check -p qbz` and grep for `artist_blacklist::` call sites (notably
  in `search.rs`, this batch's other file) to confirm the public path is
  unchanged.
