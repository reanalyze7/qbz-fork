# crates/qbz-qobuz/src/link_resolver.rs (311 lines)

## Summary
Pure, I/O-free parser that resolves Qobuz deep links (`qobuzapp://` scheme
and `https://play.qobuz.com/` / `https://open.qobuz.com/` web URLs) into a
typed `ResolvedLink` navigation action (OpenAlbum/OpenTrack/OpenArtist/
OpenPlaylist), plus a large `#[cfg(test)]` module (~165 lines) covering
happy paths, trimming/query/fragment stripping, and error cases.

## Proposed split
This file is small and entirely pure logic — a light by-concern split, not
a directory:

- `link_resolver.rs` (~95 lines) — becomes the re-export/public-API
  surface: module doc, `ResolvedLink` enum, `LinkResolverError` enum, and
  the single public `resolve_link` entry point (kept here since it's the
  one thing every caller imports), plus `#[cfg(test)] mod tests;`
  declaration pointing at the extracted test file.
- `link_resolver_parse.rs` (~50 lines) — `strip_web_prefix`,
  `parse_path_segments` (the URL-shape parsing helpers).
- `link_resolver_build.rs` (~35 lines) — `build_resolved_link` (the
  entity-type → `ResolvedLink` construction/validation match).
- `link_resolver/tests.rs` (~165 lines) — the entire `#[cfg(test)] mod
  tests` block verbatim, referencing `super::*`.

Given the whole file is only 311 lines, an equally valid (simpler)
alternative is a two-way split: keep parsing+building together in one
~180-line file and only extract `tests.rs` — reassess against the 130-line
budget once the real split is attempted; if 180 is still judged too big,
fall back to the three-way split above.

## Re-export surface
Whichever split is chosen, `link_resolver.rs` itself (not a `mod.rs` — this
can stay a flat module, no directory needed since Rust allows
`#[path]`-free sibling files declared via `mod link_resolver_parse;` etc.
from within `link_resolver.rs`) remains the target of the existing `mod
link_resolver;` in `crates/qbz-qobuz/src/lib.rs`. It must keep re-exporting
(or directly defining) `ResolvedLink`, `LinkResolverError`, and
`resolve_link` — the three symbols the doc comment and any external caller
(e.g. a deep-link handler in `qbz` or `qbzd`) would import as
`qbz_qobuz::link_resolver::{resolve_link, ResolvedLink, LinkResolverError}`.

## Coupling / watch out
- `parse_path_segments` and `build_resolved_link` are called in sequence
  from `resolve_link` and nowhere else — low risk, but keep them
  `pub(crate)` or module-private (`pub(super)`) rather than fully `pub` so
  the public surface doesn't accidentally widen.
- `ResolvedLink`/`LinkResolverError` derive `Serialize`/`Deserialize` (serde
  tag/content attrs) — if these types move to a separate file, the derive
  macros and `thiserror::Error` import must move with them; `resolve_link`
  elsewhere just needs `use super::{ResolvedLink, LinkResolverError};`.
- Zero global/shared state — purely functional, essentially zero risk split.

## Verify after split
- `cargo test -p qbz-qobuz link_resolver::` — the existing test suite is
  comprehensive (happy paths for every entity type + scheme, trimming,
  query/fragment stripping, http/open.qobuz variants, and every error
  case) and is the primary regression net; all assertions must stay
  unchanged and green.
- `cargo build` for any crate importing `qbz_qobuz::link_resolver::*`
  (grep callers) to confirm the public path is unchanged.
