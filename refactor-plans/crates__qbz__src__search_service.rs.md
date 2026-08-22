# crates/qbz/src/search_service.rs (186 lines)

## 1. Summary

A process-global lifecycle wrapper (`init`/`teardown`) around the
headless `qbz_app::settings::search_service::SearchService`, plus thin
accessor functions (`is_enabled`, `cached`, `store`, `record`,
`top_for_query`, `rank_within`) that fail-safe to "disabled" when no
session is bound, with one combined lifecycle-roundtrip test.

## 2. Proposed module split

Small file, just over budget. Split lifecycle/accessors from the test:

| New file | Owns | ~lines |
|---|---|---|
| `search_service/mod.rs` | The `SERVICE` static, `init`, `teardown`, `with_service`, `with_service_mut`, `set_enabled`, `is_enabled`, `cached`, `store`, `record`, `top_for_query`, `rank_within` — plus the `pub use ... InteractionAction` re-export and module doc comment | ~130 |
| `search_service/tests.rs` | The `#[cfg(test)] mod tests` block: `unique_temp_dir()` helper + the single `lifecycle_roundtrip` test | ~55 |

This is the minimal viable split — all the lifecycle/accessor functions
are tightly coupled through the one `SERVICE` static and are better kept
together; only the test module (which doesn't need to compile in
release builds) is pulled out.

## 3. Re-export / public API surface

`search_service/mod.rs` keeps being the whole public surface — same
function names at the same `qbz::search_service::X` path:

```rust
mod tests; // #[cfg(test)]

pub use qbz_app::settings::search_service::InteractionAction;

pub fn init(base_dir: &Path, enabled: bool) { ... }
pub fn teardown() { ... }
pub fn set_enabled(on: bool) { ... }
pub fn is_enabled() -> bool { ... }
pub fn cached(query: &str) -> Option<qbz_models::SearchAllResults> { ... }
pub fn store(query: &str, results: &qbz_models::SearchAllResults) { ... }
pub fn record(query: &str, kind: &str, id: &str, action: InteractionAction) { ... }
pub fn top_for_query(query: &str) -> Option<(String, String)> { ... }
pub fn rank_within<T>(...) { ... }
```

No caller-visible change: `qbz-slint`'s search cortinilla and the SWR
result-page controller (mentioned in the file's own doc comment) keep
calling `qbz::search_service::*` exactly as before.

## 4. Tricky coupling to watch out for

- `SERVICE: Mutex<Option<SearchService>>` is a `static` — moving it
  requires nothing special (statics don't need re-exporting, they're
  module-private already), but it's the single piece of shared state
  every accessor closes over via `with_service`/`with_service_mut` —
  keep all accessors in the same file as the static; don't split further
  than proposed or you'll need `pub(crate)` on the static and the two
  `with_service*` helpers.
- The module doc comment explicitly documents the fail-safe contract
  ("with no session bound... every accessor behaves as disabled") — this
  is important enough to keep prominently in `mod.rs`, not buried or
  lost in the split.
- The re-exported `InteractionAction` type comes from
  `qbz_app::settings::search_service` — a different crate's module of
  almost the same name (`search_service`) as this one
  (`qbz::search_service`) — worth a one-line comment noting the
  distinction (headless model crate vs. this per-session wrapper) so a
  future reader doesn't confuse the two, especially since this file's
  own doc comment already flags the naming overlap with "ADR-006: the
  cache... model logic lives in qbz-app; this module only owns the
  per-user store lifecycle."

## 5. What to verify after the real split

- `cargo build -p qbz` and `cargo test -p qbz search_service::` — the
  single `lifecycle_roundtrip` test stays green. Note: this test is
  explicitly a **single combined test** because the underlying
  `SERVICE` is a process-global singleton and parallel tests would
  clobber each other's state (per the test's own doc comment) — do not
  split it into multiple `#[test]` functions during this reorg, that
  would reintroduce the exact race the original author avoided.
- Grep for `search_service::` usages in `qbz-ui`/`qbz-slint` (the search
  cortinilla, SWR result-page controller) to confirm import paths are
  unaffected.
- Smoke-test: log in, run a search, verify cached results / ranked
  ordering behave the same as before the split (`init`/`teardown` wiring
  happens at login/logout — confirm both still fire at the right call
  sites).
