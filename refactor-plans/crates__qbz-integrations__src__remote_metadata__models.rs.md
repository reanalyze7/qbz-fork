# crates/qbz-integrations/src/remote_metadata/models.rs (238 lines)

Unified DTOs for remote metadata providers (MusicBrainz/Discogs): provider
enum, search-result/full-metadata/track structs, request/response wrappers,
error enum.

## Proposed split

- `remote_metadata/models/mod.rs` (~40 lines) — re-export surface,
  `RemoteProvider` enum + its `Display`/`FromStr` impls.
- `remote_metadata/models/album.rs` (~90 lines) — `RemoteAlbumSearchResult`,
  `RemoteAlbumMetadata`, `RemoteTrackMetadata`.
- `remote_metadata/models/request.rs` (~70 lines) — `RemoteSearchRequest`
  (+ `limit()`), `RemoteSearchResponse` (+ `empty`/`rate_limited`).
- `remote_metadata/models/error.rs` (~40 lines) — `RemoteMetadataError` +
  `Display` + `From<RemoteMetadataError> for String`.
- `remote_metadata/models/tests.rs` (~40 lines) — existing test module.

## Tricky coupling

- `RemoteProvider` is used by every other struct (as a field) — must be
  `pub` from `mod.rs` and imported by `album.rs`/`request.rs`.
- No external state/statics; purely data types, low risk split.

## Verify after split

`cargo build -p qbz-integrations`, `cargo test -p qbz-integrations
remote_metadata::models::`.
