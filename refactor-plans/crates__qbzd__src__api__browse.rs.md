# crates/qbzd/src/api/browse.rs (242 lines)

## Summary
The daemon's catalog READ routes: `GET /api/album`, `/api/artist`,
`/api/similar`, `/api/suggest` — all auth-gated, all pure reads returning
the core's typed serde shapes verbatim.

## Proposed split
This file is only marginally over 130 lines (242) once split in two —
routes vs. shared internals + tests is enough; no need for a full
`browse/` directory with many tiny files.

- `browse/mod.rs` (~120 lines) — module doc (lines 1-9), `DEFAULT_LIMIT`,
  `MAX_LIMIT`, `QUEUE_SEED_CAP` consts, all 4 route fns: `album`, `artist`,
  `similar`, `suggest`.
- `browse/query.rs` (~55 lines) — internals: `parse` (query-string
  decoder), `limit_offset`, `wants` (boolean-flag reader).
- `browse/errors.rs` (~15 lines) — `auth_gate`, `not_found` (shared
  response builders).
- `browse/tests.rs` (~35 lines) — the `#[cfg(test)] mod tests` block
  (`parse_decodes_values_and_splits_pairs`, `limit_offset_clamps_and_defaults`,
  `wants_reads_truthy_flags`), declared via `#[cfg(test)] mod tests;` in
  `mod.rs`.

## Re-export surface
`browse/mod.rs` re-exports `album`, `artist`, `similar`, `suggest` at
`crate::api::browse::*` — the `crates/qbzd/src/api/mod.rs` router keeps
dispatching to the same path unchanged.

## Coupling / watch out
- `auth_gate` here is structurally identical to `playback.rs`'s own private
  `auth_gate` (see `refactor-plans/crates__qbzd__src__api__playback.rs.md`)
  — this is a duplicated-but-independent helper across two files in the
  same chunk. Do NOT merge them into one shared `api::errors::auth_gate`
  during this mechanical split (that's a separate, larger refactor
  decision); just relocate each file's own copy into its own
  `browse/errors.rs` / `playback/errors.rs`. Flagging this for whichever
  agent/person eventually looks at `api/mod.rs` as a whole — a shared
  `super::auth_gate` helper in `api/mod.rs` itself would remove this
  duplication across at least these two files.
- `parse`, `limit_offset`, `wants` are called from all 4 route fns in
  `mod.rs` — straightforward `use super::query::{parse, limit_offset,
  wants};` import, no shared mutable state (pure functions over a
  `HashMap<String, String>`).
- `not_found` is called from `album`/`artist`/`similar` — same trivial
  `use super::errors::not_found;`.

## Verify after split
- `cargo test -p qbzd api::browse::` — all 3 tests green.
- `cargo check -p qbzd` and grep for `crate::api::browse::{album, artist,
  similar, suggest}` importers in `crates/qbzd/src/api/mod.rs` to confirm
  the public path is unchanged.
