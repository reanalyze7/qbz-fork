# crates/qbz-app/src/settings/search_service.rs (241 lines)

## Summary
Thin `SearchService` facade (ADR-006, per the sibling `qbz/src/
search_service.rs` re-export doc comment) wrapping a search cache +
interaction ranking: `enabled`/`set_enabled`, `cached`/`store`,
`record_interaction`, `top_for_query`, and a generic `rank_within<T>`
helper. Only ~110 lines over budget, mostly due to its test module.

## Proposed split
- `search_service/mod.rs` (~40 lines) — the file's extensive module doc
  comment (lines 1-28: what SearchService is/isn't, interior-mutability
  rationale — keep this, it's load-bearing design documentation) plus
  `pub use super::search_ranking::InteractionAction;` (the re-export so
  qbz-slint imports the action enum from one place, per its own comment).
- `search_service/service.rs` (~95 lines) — `SearchService` struct +
  impl: `new`, `enabled`, `set_enabled`, `cached`, `store`,
  `record_interaction`, `top_for_query`, `rank_within` (lines 42-136).
- `search_service/tests.rs` (~100 lines) — the `#[cfg(test)] mod tests`
  block (lines 137-241), including its fixture builders (`page`, `album`,
  `track`, `playlist`, `artist`, `sample_results`).

Given the modest size, a straight 2-way split (impl vs tests) is
sufficient — no need for finer sectioning of the service methods
themselves.

## Re-export surface
`search_service/mod.rs` re-exports `SearchService` and
`InteractionAction` (re-exported here FROM `super::search_ranking`, per
the file's own "re-export so qbz-slint imports the action enum from ONE
place" comment) at the current `qbz_app::settings::search_service::X`
path. Confirmed external caller: `crates/qbz/src/search_service.rs` does
`use qbz_app::settings::search_service::SearchService;` and `pub use
qbz_app::settings::search_service::InteractionAction;` — both names must
keep resolving here exactly as today.

## Coupling / watch out
- `InteractionAction` is NOT defined in this file — it's a re-export from
  `super::search_ranking::InteractionAction`. Keep that `pub use` line
  (not the enum definition) wherever the split lands, don't try to move a
  definition that doesn't exist here.
- `SearchService` is deliberately non-generic (does not hold `QbzCore<A>`)
  — the module doc explains this at length; preserve the doc comment
  verbatim so a future contributor doesn't "fix" this by adding a generic.
- `rank_within<T>` is generic — confirm its trait bounds/imports move
  cleanly into `service.rs` without needing anything from the (deleted)
  monolithic file scope.
- This module explicitly delegates cache mechanics to
  `search_cache.rs` (ADR-006 boundary) — do not merge the two files back
  together even though they're conceptually related; they're already
  split at the crate-file level intentionally.

## Verify after split
- `cargo test -p qbz-app settings::search_service` green.
- `cargo build -p qbz` (the `search_service.rs` re-export) and `-p
  qbz-app`.
