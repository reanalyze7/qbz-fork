# crates/qbz-integrations/src/discogs/mod.rs (626 lines)

## Summary
`DiscogsClient`: HTTP client (via a Cloudflare Workers proxy) for Discogs
album-artwork search/download and full release-metadata lookup (used by both
the artwork picker and the tag editor), plus all the public/internal DTOs
(`SearchResponse`, `DiscogsReleaseMetadata`, `DiscogsImageOption`, etc.).

## Proposed split
Split DTOs from client logic, and split client logic by "artwork" vs.
"metadata/search" concern:

- `discogs/mod.rs` (~40 lines) — module doc, `DISCOGS_PROXY_URL` const,
  `DiscogsClient` struct + `new`/`has_credentials`/`Default`, `pub use` of
  types from `types.rs` and the `impl` blocks from the other files (multiple
  `impl DiscogsClient` blocks across files is fine within one crate).
- `discogs/types.rs` (~100 lines) — every DTO: `SearchResponse`,
  `SearchResult`, `DiscogsImageOption`, `ReleaseDetails` (+ `#[allow(dead_code)]`),
  `ReleaseImage`, `DiscogsReleaseMetadata`, `DiscogsArtist`, `DiscogsLabel`,
  `DiscogsTrack`, `DiscogsSearchResultExtended` (lines 19-128). Pure data,
  zero behavior — the natural "data model" module.
- `discogs/artwork.rs` (~230 lines) — `fetch_artwork`, `search_release`
  (private), `get_release_details` (private), `search_artwork_options`,
  `download_artwork_from_url`, `download_image` (private) — the whole
  artwork-search-and-download path (the artwork picker's backend), which
  shares `get_release_details`/`download_image` between callers.
- `discogs/search.rs` (~100 lines) — `search_artist`, `search_releases` (with
  its embedded local `ExtendedSearchResponse` struct) — the tag-editor search
  surface.
- `discogs/metadata.rs` (~40 lines) — `get_release_metadata` (full release +
  tracklist fetch, used by the tag editor after a search hit).
- `discogs/hash.rs` (~15 lines) — `simple_hash` (the DJB2-style filename
  hasher used by both `fetch_artwork` and `download_artwork_from_url`) — tiny
  but pure, worth its own file since it's the one non-networked helper.
- `discogs/tests.rs` (~14 lines) — the existing `#[cfg(test)] mod tests`
  block (just `test_hash`), declared via `#[cfg(test)] mod tests;` in
  `mod.rs`.

## Re-export surface
`discogs/mod.rs` re-exports every DTO from `types.rs` via `pub use
types::*;` and keeps `DiscogsClient` as the single client type — external
callers continue to `use qbz_integrations::discogs::DiscogsClient` (and the
DTOs) unchanged; only the module became a directory instead of one file.

## Coupling / watch out
- `simple_hash` is called from BOTH `fetch_artwork` (artwork.rs) and
  `download_artwork_from_url` (also artwork.rs, so no cross-file call
  needed there) — but if `download_artwork_from_url` stays in artwork.rs (as
  planned above) it can just call `Self::simple_hash` normally as a
  `pub(super)` or `pub(crate)` associated fn in `hash.rs`.
- `get_release_details` (in artwork.rs) returns the internal `ReleaseDetails`/
  `ReleaseImage` types (types.rs) — no cycle, just make sure `artwork.rs`
  imports them from `super::types::*`.
- The local `ExtendedSearchResponse` struct inside `search_releases` is
  defined INLINE in the function body today (lines 512-515) — keep it there
  (or promote it to `types.rs` if the implementer prefers), it's private and
  only used by that one function.
- All client methods share `self.client` (reqwest) — no shared mutable
  state/locks in this file (unlike the ListenBrainz client), so the split is
  lower-risk; each `impl DiscogsClient` block in a new file just needs
  `self.client` visible, which it is since it's a `pub` struct field... check
  whether `client` field is actually `pub` or crate-private — it's declared
  without `pub` (`pub struct DiscogsClient { client: Client }`), so it's
  private to the `discogs` module. Splitting into `discogs/artwork.rs` etc.
  (still inside the `discogs` module tree) keeps this working, but if any
  new file is NOT a descendant of the `discogs` module (i.e. not declared via
  `mod` from `discogs/mod.rs`) it will fail to compile — make sure all new
  files are declared as `mod artwork; mod search; mod metadata; mod hash;`
  from `discogs/mod.rs` so they're privacy-scoped correctly.

## Verify after split
- `cargo test -p qbz-integrations discogs::tests::test_hash` stays green.
- `cargo check -p qbz-integrations` and grep for `discogs::DiscogsClient` /
  `discogs::Discogs*` importers (likely the tag editor and artwork-picker UI
  glue in `qbz-app`) to confirm the public path and every DTO name are
  unchanged.
