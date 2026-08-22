# crates/qbz-log/src/redact.rs (144 lines)

## Summary
Write-time secret redaction: a two-layer scrubber (exact-string
replacement of runtime-registered live secrets, then a fixed regex set for
labeled-key patterns) applied to every log line via `redact()`.

## Proposed split
This file is only 14 lines over budget (144 total, ~35 of which are
tests) — the cleanest split separates the compiled-pattern/registry
statics from the `redact()` logic and its tests, keeping each cohesive:

- `redact.rs` (~75 lines) — KEEP as the main file: module doc (lines
  1-12), constants (`REPLACEMENT`, `MIN_SECRET_LEN`, lines 18-20), and the
  public API: `register_secret` (lines 63-72) and `redact` (lines 82-107).
  This is the small, cohesive "what callers use" surface.
- `redact/patterns.rs` (~40 lines) — `patterns()` (the `OnceLock<Vec<Regex>>`
  of the 10 labeled-key regexes, lines 24-53) and `has_redaction_candidate`
  (lines 76-79) — the compiled-pattern-set concern, isolated from the
  registry and the top-level `redact()` orchestration.
- `redact/registry.rs` (~15 lines) — `secrets()` (the `OnceLock<RwLock<Vec
  <String>>>` live-secret store, lines 55-58) — tiny, but conceptually
  distinct storage (runtime-registered literal secrets vs compiled static
  patterns).
- Tests (lines 109-144) stay as `#[cfg(test)] mod tests` at the bottom of
  the main `redact.rs` — they exercise `redact()`/`register_secret()`
  directly (the public API), not the internals, so they belong with the
  file that still defines those two functions.

## Re-export surface
`redact.rs` stays the only import surface: `pub use patterns::*;`
(probably not needed — `patterns()`/`has_redaction_candidate` are private
implementation details, not part of the public API) and `pub(crate) use
registry::secrets;` if `secrets()` needs to stay reachable from
`redact.rs`'s own `redact`/`register_secret` functions. The crate's
`lib.rs` line `pub mod redact;` (or however `qbz_log::redact` is exposed)
needs no change. `qbz_log::redact::{redact, register_secret}` — the only
two functions used externally (per `crate::qbz_log::register_secret`
calls seen in `login.rs` in this same batch) — are unaffected.

## Coupling / watch out
- `redact()` calls BOTH `secrets()` (registry.rs) and `patterns()` +
  `has_redaction_candidate()` (patterns.rs) in sequence — this is the
  file's central orchestration logic; keep the two-layer ORDER (literal
  live-secret replacement FIRST, then regex) exactly as documented in the
  module doc comment (lines 3-9) — reordering would change behavior (a
  live secret matching part of a regex pattern could be redacted
  differently depending on order).
- `register_secret`'s `MIN_SECRET_LEN` guard and dedup-check
  (`!guard.iter().any(|s| s == &value)`) both live in the main file
  alongside the constant it reads — no cross-file coupling issue if
  `MIN_SECRET_LEN` stays in `redact.rs` and `register_secret` is also kept
  there (as planned) rather than moved to `registry.rs`.
- This module is described in its own doc comment as "the single most
  important safety layer in this crate" — when splitting, be extra careful
  that `cargo test` actually re-runs and passes the exact same 3 tests
  (`redacts_all_known_shapes`, `literal_registry_scrubs_unlabeled_value`,
  `short_secret_is_ignored`) with no behavioral drift; this is a
  security-sensitive file, not just an organizational one.
- `qbz_log::register_secret` is called from `login.rs` (also in this
  batch) at two sites (`validate_token`, `finalize`) to register the
  user's OAuth token before it can appear in any log line — confirm this
  external call path is unaffected (it will be, since `register_secret`'s
  signature/location in `redact.rs` doesn't change).

## Verify after split
- `cargo build -p qbz-log`.
- `cargo test -p qbz-log redact::` — all 3 existing tests must still pass
  unchanged, with special attention to `redacts_all_known_shapes` (covers
  every regex pattern shape) since this is the security-critical path.
- `cargo clippy -p qbz-log`.
- Smoke-test importers: `grep -rn "qbz_log::register_secret\|qbz_log::
  redact" crates` — confirm `qbzd/src/login.rs` and any log-formatting
  layer that calls `redact()` on outgoing log lines still compile and
  behave identically (manually trigger a login flow with `RUST_LOG=debug`
  and confirm no token/secret appears in the daemon log output).
