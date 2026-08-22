# crates/qbz-models/src/events.rs (134 lines)

Just barely over 130 — a single `CoreEvent` enum with ~26 variants grouped
by comment banners (Playback/Queue/Auth/Library/Loading/Error/Audio/Search/
Navigation).

## Proposed split

This is a single cohesive enum (serde tag-based); splitting it into
multiple files would force either a giant `pub enum` spanning modules
(not idiomatic Rust) or artificially breaking one flat type. Recommend
instead:

- Trim doc-comment blank lines / tighten spacing between variant groups to
  get under 130 without changing structure (mechanical, ~10-15 lines
  saved) — OR
- If a real split is wanted: extract the doc comments on each variant
  group into a short module-level table (reduces line count) while keeping
  `CoreEvent` itself intact in `events.rs`.

No sub-file split recommended; this is a "trim, don't fragment" case.

## Tricky coupling

- `CoreEvent` is the crate's primary cross-cutting type — every frontend
  adapter matches on it. Do not split the enum across files.

## Verify after any trim

`cargo build -p qbz-models`, `cargo test -p qbz-models`.
