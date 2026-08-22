# crates/qbz-integrations/src/remote_metadata/mod.rs (388 lines)

## 1. Summary

Pure converter functions turning MusicBrainz/Discogs API response types
into the crate's unified `RemoteAlbumSearchResult`/`RemoteAlbumMetadata`
DTOs (re-exported from a sibling `models` module) — no I/O, all pure data
transformation, frontend-agnostic copies of logic that also lives in the
Tauri app.

## 2. Proposed module split

The file already has a clean provider boundary (`// ==== Discogs Adapter
====` marks the halfway point) plus a search-result-vs-full-metadata
axis. Split by provider, each provider file holding both of its
converters:

| New file | Owns | ~lines |
|---|---|---|
| `remote_metadata/mod.rs` | Module decls, `pub use models::{...}` re-export (unchanged), `pub use` of the converter functions | ~20 |
| `remote_metadata/musicbrainz.rs` | `musicbrainz_release_to_search_result`, `musicbrainz_full_to_metadata` | ~150 |
| `remote_metadata/discogs.rs` | `parse_discogs_position`, `parse_discogs_duration`, `discogs_extended_to_search_result`, `discogs_full_to_metadata` | ~145 |
| `remote_metadata/models.rs` | Unchanged — already a separate module (`mod models;` at top of current file) | (existing) |

## 3. Re-export / public API surface

`remote_metadata/mod.rs` keeps the exact same public surface:

```rust
mod discogs;
mod models;
mod musicbrainz;

pub use models::{
    RemoteAlbumMetadata, RemoteAlbumSearchResult, RemoteMetadataError, RemoteProvider,
    RemoteSearchRequest, RemoteSearchResponse, RemoteTrackMetadata,
};
pub use discogs::{
    discogs_extended_to_search_result, discogs_full_to_metadata,
    parse_discogs_duration, parse_discogs_position,
};
pub use musicbrainz::{musicbrainz_full_to_metadata, musicbrainz_release_to_search_result};
```

Every caller currently doing
`qbz_integrations::remote_metadata::musicbrainz_release_to_search_result(...)`
(or similar) keeps working unchanged.

## 4. Tricky coupling to watch out for

- Both provider files reference sibling-crate types by fully qualified
  path (`crate::musicbrainz::ReleaseResult`, `crate::discogs::DiscogsSearchResultExtended`,
  etc.) — these are the **actual API client types** living elsewhere in
  `qbz-integrations` (`crate::musicbrainz`, `crate::discogs` modules, not
  to be confused with the new `remote_metadata/discogs.rs` /
  `musicbrainz.rs` files being proposed here). Naming collision risk:
  the new file `remote_metadata/discogs.rs` and the existing
  `crate::discogs` module are different modules with similar names —
  keep the `crate::discogs::...` qualified paths in the new file
  (`remote_metadata::discogs`) unambiguous; Rust's module system handles
  this fine since they're different paths, but a future reader could
  easily confuse "the discogs converter module" with "the discogs API
  client module" — worth a one-line doc comment at the top of each new
  file clarifying which is which.
- Both converters independently duplicate the same "artist from
  artist-credit + joinphrase" concatenation logic
  (`musicbrainz_release_to_search_result` and `musicbrainz_full_to_metadata`
  both do it) and the same "label/catalog-number from first label-info
  entry" pattern — this duplication already exists in the current file
  and is preserved as-is by a pure file-split; do NOT extract a shared
  helper as part of this move (that's a separate simplification, not a
  130-line-rule split) unless doing so is trivial and risk-free.
- `models.rs` is already a separate file/module — confirm its current
  line count is also within budget (not in this agent's assigned list;
  note for whoever does the actual split to check it too).

## 5. What to verify after the real split

- `cargo build -p qbz-integrations` and `cargo test -p qbz-integrations
  remote_metadata::` — this file currently has no visible `#[cfg(test)]`
  block itself (tests likely live elsewhere or are absent); confirm
  during the real split whether any doctest/unit test references these
  functions and keep them passing.
- Grep the workspace for
  `remote_metadata::{musicbrainz_*, discogs_*, parse_discogs_*}` usages
  (likely in `qbz-app` or `qbz-ui` album-lookup controllers) to confirm
  no import path broke.
- Since these are pure functions, a quick unit-test smoke (feed a
  representative MusicBrainz/Discogs JSON fixture through each converter)
  is cheap insurance if none currently exists — consider adding one per
  the project's "tests every time" rule if this crate currently has none
  for this module.
