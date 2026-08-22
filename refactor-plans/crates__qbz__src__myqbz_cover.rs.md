# crates/qbz/src/myqbz_cover.rs (232 lines)

## Summary
My QBZ custom-cover upload/remove: file-picker → validate extension →
decode/resize(1000×1000 Lanczos3)/save as JPEG → persist path via
`qbz_mixtape::repo` → delete previous file (persist-before-delete
ordering), plus a detail-view reload after either action.

## Proposed split
By concern — pure blocking helpers vs the two public async entry points:

- `myqbz_cover/mod.rs` (~50 lines) — `ALLOWED_EXTENSIONS`, `epoch_secs`,
  `safe_id`, `pub use` of submodules.
- `myqbz_cover/db.rs` (~35 lines) — `get_prev_path`, `set_custom_artwork`
  (the two `library_db::with_db` wrappers).
- `myqbz_cover/image_io.rs` (~15 lines) — `resize_and_save`.
- `myqbz_cover/ops.rs` (~65 lines) — `do_upload`, `do_remove` (the two
  blocking operation bodies, each calling into `db.rs`/`image_io.rs`).
- `myqbz_cover/actions.rs` (~75 lines) — `upload`, `remove`, `reload` (the
  public async entry points + shared reload helper).

## Re-export surface
`myqbz_cover/mod.rs` stays the `mod myqbz_cover;` target. `upload` and
`remove` (the only two functions called from `main.rs`'s hero-overflow
menu) re-exported via `pub use actions::{upload, remove};` — unchanged
call sites.

## Coupling / watch out
- **Persist-before-delete ordering is the single most important invariant
  in this file** (`do_upload`'s comment: "persist BEFORE deleting the
  previous file, and only delete when it differs" — mirrors the Tauri
  command step order exactly). Keep `do_upload`'s 9 numbered steps in one
  function, in order; do not split step 8 (persist) and step 9 (delete
  prev) into different files/functions where their sequencing could get
  lost.
- On a persist failure after a successful resize/save, `do_upload` deletes
  the ORPHAN file it just wrote (`std::fs::remove_file(&dest)`) before
  returning the error — this cleanup-on-failure branch is easy to drop
  during a split; preserve it.
- The webp note (workspace `image` crate has no webp feature enabled, so a
  `.webp` source decodes to a runtime error despite being accepted by the
  extension filter) is a known, documented limitation — don't "fix" it as
  part of a mechanical split; if anything, flag it for the real split PR to
  decide whether to drop `.webp` from `ALLOWED_EXTENSIONS`/the picker
  filter or actually enable the feature.
- `reload()` calls `crate::myqbz_detail::global_runtime()` and
  `crate::myqbz_detail::navigate(...)` — cross-module coupling to the
  detail view; keep the `crate::myqbz_detail::` paths intact wherever
  `reload` lands (per this plan, `actions.rs`).
- `safe_id`'s comment notes the collection id is normally already a UUID
  so the sanitization is "normally a no-op" — defensive code, not currently
  exercised by any test; a real split PR could add a unit test for it.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap;
  `safe_id` and `epoch_secs`-adjacent filename building are good unit-test
  candidates, and `do_upload`/`do_remove`'s persist-then-delete ordering
  would benefit from an integration test against a temp `library.db` +
  temp artwork dir in a real split PR).
- Manual test: upload a custom cover (png/jpg), confirm the detail hero
  updates and the OLD file is deleted; remove the custom cover, confirm it
  reverts and the file is deleted; try a `.webp` file and confirm it
  produces the expected "upload failed" toast (documented limitation).
