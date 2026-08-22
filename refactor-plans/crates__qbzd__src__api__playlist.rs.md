# crates/qbzd/src/api/playlist.rs (216 lines)

## Summary
`qbzd` HTTP API handlers for playlists: `GET /api/playlists` (list),
`GET /api/playlist?id=` (show, full track list), and CRUD/track-mutation
endpoints (`create`, `update`, `delete`, `tracks_add`, `tracks_remove`), all
auth-gated via `AuthState`, plus small query/body-parsing internals and unit
tests.

## Proposed split
Split by read vs. write endpoints, keeping the small shared internals
together (they're used by every handler).

- `playlist/mod.rs` (~15 lines) — module doc/header comment, `pub use`
  re-exports of every public fn (`list`, `show`, `create`, `update`,
  `delete`, `tracks_add`, `tracks_remove`) so the router in `api/mod.rs` (or
  wherever routes dispatch to `playlist::list`/`playlist::show`/etc.) is
  unaffected.
- `playlist/reads.rs` (~35 lines) — `list`, `show` (the two GET handlers).
- `playlist/crud.rs` (~50 lines) — `create`, `update`, `delete` (the three
  playlist-level POST handlers).
- `playlist/tracks.rs` (~65 lines) — `tracks_add`, `tracks_remove` (the two
  track-mutation handlers; `tracks_remove` is the most complex — it resolves
  plain track ids to per-playlist `playlist_track_id` row ids before calling
  core).
- `playlist/internal.rs` (~45 lines) — `parse_ids`, `id_param`, `auth_gate`
  (shared helpers every handler above calls).
- `playlist/tests.rs` (~20 lines) — the `#[cfg(test)] mod tests` block
  (`id_param_reads_numeric_id`, `parse_ids_accepts_valid_and_rejects_bad`).

## Re-export surface
`playlist/mod.rs` re-exports `list`, `show`, `create`, `update`, `delete`,
`tracks_add`, `tracks_remove` at `crate::api::playlist::*` — the HTTP route
table in `qbzd` (likely `crates/qbzd/src/api/mod.rs` or `main.rs`) dispatches
by calling `playlist::list(&state)` etc.; those call sites are unaffected as
long as `mod.rs` re-exports every handler at the same path.

## Coupling / watch out
- Every handler starts with `if let Some(resp) = auth_gate(state) { return
  resp; }` — keep `auth_gate` visible (`pub(super)` or re-exported) to every
  submodule that calls it.
- `tracks_remove` is the most tightly coupled handler: it calls
  `parse_ids`, `auth_gate`, AND does its own `get_playlist` fetch + row-id
  resolution before calling `remove_tracks_from_playlist` — keep the whole
  function together in `tracks.rs`, do not split its internals across files.
- `ApiState`, `err_json`, `json` come from `super::{err_json, json,
  ApiState}` (the parent `api` module) — every new submodule file needs the
  same `use super::{err_json, json, ApiState};` (or `use
  crate::api::{...}` if that's cleaner) plus `use crate::state::AuthState;`.
- `Cursor<Vec<u8>>` / `tiny_http::Response` return type appears on every
  handler — no special handling needed, just repeat the `use` imports per
  file.

## Verify after split
- `cargo test -p qbzd api::playlist::` (or wherever the module path lands)
  — both existing tests (`id_param_reads_numeric_id`,
  `parse_ids_accepts_valid_and_rejects_bad`) green.
- `cargo check -p qbzd` and confirm the route-dispatch table (grep for
  `playlist::list`/`playlist::show`/`playlist::create` etc. in
  `crates/qbzd/src/api/mod.rs` or the main request router) still resolves.
- Manual/smoke: start `qbzd`, exercise `GET /api/playlists`,
  `GET /api/playlist?id=<id>`, and a create/update/delete/tracks round-trip
  with `curl` to confirm the auth gate and JSON error shapes are unchanged.
