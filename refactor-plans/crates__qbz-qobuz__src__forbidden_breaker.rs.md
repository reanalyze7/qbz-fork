# crates/qbz-qobuz/src/forbidden_breaker.rs (159 lines)

## Summary
A 403-response circuit breaker (issue #637): after N consecutive HTTP 403s
from Qobuz it opens for an exponentially-growing cooldown so the prefetch
scheduler backs off instead of hammering the API into an IP-level WAF ban;
a single post-cooldown probe either closes it (success) or re-opens it
(another 403) with a longer cooldown.

## Proposed split

- `forbidden_breaker/mod.rs` (~65 lines) — module doc, the tuning constants
  (`OPEN_THRESHOLD`, `BASE_COOLDOWN`, `MAX_COOLDOWN`), `Inner` struct,
  `ForbiddenBreaker` struct + `Default` impl + `pub use` of nothing extra
  needed (the struct itself lives here).
- `forbidden_breaker/breaker.rs` (~55 lines) — `impl ForbiddenBreaker`:
  `new`, `blocked_for`, `record_success`, `record_forbidden` — the actual
  state-machine logic, split out from the struct definitions so `mod.rs`
  stays a thin declarations file.
- `forbidden_breaker/tests.rs` (~55 lines) — the entire `#[cfg(test)] mod
  tests` block (`stays_closed_below_threshold`, `opens_at_threshold_and_blocks`,
  `success_resets_everything`, `cooldown_grows_exponentially_and_caps`).

Given the file is only 29 lines over budget, an even simpler two-way split
(impl+tests) may suffice if the reviewer prefers fewer files:
- `forbidden_breaker.rs` (~100 lines) — everything except tests.
- `forbidden_breaker/tests.rs` (~55 lines) — tests only, via
  `#[cfg(test)] mod tests;` at the bottom of `forbidden_breaker.rs` (this
  requires `forbidden_breaker.rs` to become `forbidden_breaker/mod.rs` OR use
  the `#[path]` attribute — simplest is converting the single file into a
  `forbidden_breaker/` directory with `mod.rs` + `tests.rs`, consistent with
  the rest of this convention).

## Re-export surface
`forbidden_breaker/mod.rs` re-exports `ForbiddenBreaker` (the only public
type) — `qbz_qobuz::forbidden_breaker::ForbiddenBreaker` (or however the
crate's lib.rs currently re-exports it, e.g. `qbz_qobuz::ForbiddenBreaker` if
flattened at the crate root) stays unchanged.

## Tricky coupling / watch out
- `Inner` is a private struct only ever touched through
  `self.inner.lock().unwrap_or_else(|p| p.into_inner())` — this
  poison-recovery pattern appears in all three `ForbiddenBreaker` methods;
  keep it identical in each (don't let one method "helpfully" switch to
  plain `.unwrap()` during the move).
- `next_cooldown`'s doubling-and-capping (`(g.next_cooldown * 2).min(MAX_COOLDOWN)`)
  happens in `record_forbidden` regardless of whether THIS call opened the
  breaker or not — i.e., it grows even while already open (each renewed 403
  during an open window still doubles the cooldown for the next open) —
  this is exercised by the `cooldown_grows_exponentially_and_caps` test and
  must not be "simplified" during the split.
- `consecutive` is deliberately NOT reset when the breaker opens (only
  `record_success` resets it) — the comment on the `Inner.consecutive` field
  explains this is intentional so a single further 403 immediately re-opens
  post-cooldown. Keep this comment attached to the field wherever it lands.
- This breaker is meant to be constructed once and shared (likely behind an
  `Arc` at the call site in the prefetch scheduler) — the split doesn't
  change ownership, just confirm no caller was relying on the impl block
  being in the same file as the struct for private-field access (they're all
  in the same crate module, so this is a non-issue, just noting for
  completeness).

## What to verify after the real split
- `cargo test -p qbz-qobuz forbidden_breaker` — all four tests green.
- `cargo build -p qbz-qobuz` and grep for `ForbiddenBreaker` across
  `crates/qbz-qobuz/src/` (expected in the streaming/favorites request paths
  per the module doc) to confirm construction/usage sites are unaffected.
- No dedicated smoke test is practical (requires triggering real 403s); rely
  on the unit tests plus a code-read confirmation that the prefetch
  scheduler's call sites (`blocked_for`/`record_success`/`record_forbidden`)
  still compile against the same method signatures.
