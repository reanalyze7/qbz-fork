# crates/qbzd/src/api/fav.rs (195 lines)

## Summary
Favorites HTTP routes: `GET /api/favorites` (paged), `POST
/api/favorites/add|remove`. Hides the Qobuz plural-read/singular-write
`fav_type` inconsistency behind a singular-everywhere contract via the local
`FavType` enum.

## Proposed split
- `mod.rs` (~75 lines) — `list`, `add`, `remove`, `mutate` (the route
  handlers).
- `fav_type.rs` (~35 lines) — `FavType` enum + `parse`/`singular`/`plural`,
  `bad_type`.
- `auth.rs` (~15 lines) — `auth_gate` (shared with other `api/*.rs` files —
  see coupling note).
- `query.rs` (~15 lines) — `pairs` (query-string percent-decode helper).
- `tests.rs` (~55 lines) — existing `#[cfg(test)] mod tests`.

## Re-export surface
`mod.rs` (i.e. `crate::api::fav`) keeps exporting `list`, `add`, `remove` —
the only functions called from `crate::api::mod.rs`'s route table.

## Coupling / watch-outs
- `auth_gate` here is a near-duplicate of `auth_gate` in `api/playback.rs`,
  `api/queue.rs`, and `api/browse.rs` (each currently has its own private
  copy, per this file's own comment convention). If those files are also
  being split, consider consolidating into one shared
  `crate::api::auth::auth_gate` in a future pass — but that's a behavior-
  identical refactor beyond a pure line-count split, so do it as a
  deliberate follow-up, not silently during this split.
- `FavType::plural()`/`singular()` encode the read-vs-write Qobuz API
  quirk — keep the doc comment on the enum verbatim when moved.

## Verify after split
`cargo test -p qbzd api::fav::`; smoke-test `qbzd fav list`/`add`/`remove`
CLI commands still round-trip correctly against the HTTP API.
