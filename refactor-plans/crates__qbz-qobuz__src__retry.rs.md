# crates/qbz-qobuz/src/retry.rs (199 lines)

Transient-error retry helper for the streaming fetch path: `FetchError`
classification + exponential-backoff retry loop.

## Proposed split

Modest overage (199 lines, ~75 of which are tests). A light split covers
it:

- `retry/mod.rs` (~110 lines) — re-export surface: `DEFAULT_MAX_ATTEMPTS`,
  `FetchError` (+ impls), `reqwest_is_transient`, `classify_reqwest`,
  `classify_status`, `backoff_delay`, `retry_transient`.
- `retry/tests.rs` (~75 lines) — move `#[cfg(test)] mod tests` out,
  included via `#[path = "tests.rs"] mod tests;`.

No further functional split needed — this is one small, cohesive retry
utility; fragmenting `FetchError`/`retry_transient` further would hurt
readability more than help.

## Tricky coupling

- `retry_transient` is generic over `F/Fut/T/E` and used directly by
  `crates/qbz-qobuz/src/cmaf.rs`'s `fetch_bytes_with_retry` — keep the
  public signature and `pub` visibility unchanged.

## Verify after split

`cargo build -p qbz-qobuz`, `cargo test -p qbz-qobuz retry::` (4 existing
tests: succeeds-first-try, retries-then-succeeds, terminal-no-retry,
gives-up-after-max).
