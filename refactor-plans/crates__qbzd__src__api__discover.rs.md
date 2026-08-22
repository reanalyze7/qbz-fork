# `crates/qbzd/src/api/discover.rs` (161 lines)

## 1. Summary
The `qbzd` HTTP daemon's `GET /api/discover` handler: a `section=`-selector
dispatch over the Qobuz-backed discover family (index/most-streamed/
new-releases/press-awards/qobuzissims/album-of-the-week/ideal-discography/
playlists/tags/release-watch/featured), plus small query-parsing helpers
(`pairs`, `get`, `parse_genre`, `limit_offset`) and their unit tests.

## 2. Proposed module layout

Convert to `discover/` directory (mirrors `pub mod discover;` already
declared in `api/mod.rs`, so this is a drop-in directory-for-file swap):

- `discover/mod.rs` (~90) — the module doc comment (route table), imports,
  `DEFAULT_LIMIT`/`MAX_LIMIT` consts, and the `pub fn discover(state, query)`
  handler itself (the big `match section.as_str() { ... }` dispatch) plus
  `serialize()` and `auth_gate()`. **This is the re-export/public-API
  surface** — `api/mod.rs`'s `pub mod discover;` and any caller of
  `discover::discover(...)` need zero changes.
- `discover/query.rs` (~55) — `pairs()`, `get()`, `parse_genre()`,
  `limit_offset()` — the generic query-string parsing helpers, none of
  which are discover-specific in principle (worth a follow-up note: these
  look reusable across other `qbzd` API route handlers, but that's out of
  scope for this mechanical split — just extract them as-is).
- `discover/tests.rs` (~20) — the existing `#[cfg(test)] mod tests`
  (`parse_genre_reads_csv_or_none`, `get_and_limit_offset_defaults`),
  referenced from `discover/mod.rs` via `#[cfg(test)] mod tests;`, or kept
  inline at the bottom of `query.rs` since that's what it actually tests —
  **prefer inlining into `query.rs`** (co-location with the tested code) to
  avoid a near-empty extra file.

Net result: `discover/mod.rs` (~90) and `discover/query.rs` (~75 incl.
tests), both comfortably under 130 — this file barely needed splitting at
all; the two-file split above is the minimum viable one.

## 3. Re-export / public API surface
`api/mod.rs` already does `pub mod discover;` and presumably calls
`discover::discover(state, query)` (or similar) from the request router —
converting `discover.rs` to `discover/mod.rs` requires no change to that
call site at all, since Rust resolves `mod discover;` to either
`discover.rs` or `discover/mod.rs` transparently.

## 4. Tricky coupling to watch
- `discover()` calls `state.rt.block_on(core.get_discover_*(...))` for
  every section — these are all calls into `qbz_core::CoreError`-returning
  async methods; the `serialize()` helper centralizes the
  Result-to-JSON-or-502 mapping. Keep `serialize()` in `mod.rs` next to
  `discover()` since it's tightly coupled to that one function's error
  contract (502 `discover_failed` for every upstream error, uniformly) —
  don't move it into `query.rs` (that file is purely about request PARSING,
  not response mapping).
- `auth_gate()` reads `state.shared.lock()` — a `Mutex` shared with the rest
  of the daemon's state; it's a one-liner today but touches shared
  mutable state directly, so keep it in `mod.rs` where the rest of the
  handler's control flow is visible, rather than hiding it in the
  query-parsing file.
- `parse_genre` is called both from `discover()`'s main dispatch (as
  `genre`) and again inline for the `"featured"` arm (`genre.as_ref()
  .and_then(|v| v.first().copied())`) — no special coupling risk, just note
  both call sites need `use super::query::parse_genre;` (or equivalent)
  after the split.
- The section-name string literals (`"index"`, `"most-streamed"`, etc.) are
  duplicated between the `match` arms and the error message's help text
  listing valid sections — if these ever drift (e.g. a new section added to
  the match but not the help text, as already happened: `"featured"` is
  handled but missing from the printed section list) that's a pre-existing
  bug, not something this split should silently "fix" — note it for the
  team but leave behavior unchanged unless asked.

## 5. What to verify after the real split
- `cargo test -p qbzd` — `parse_genre_reads_csv_or_none` and
  `get_and_limit_offset_defaults` stay green.
- `cargo build -p qbzd` and `cargo build --workspace`.
- Smoke-test the actual daemon route (`qbzd`'s existing manual/integration
  test flow, e.g. `curl localhost:PORT/api/discover?section=index` against
  a logged-in daemon, or whatever the project's existing smoke-test
  convention is) to confirm the route still dispatches correctly for at
  least `index`, one Qobuz-album section, `playlists`, and `featured`
  (the one with its own 400 error branch).
