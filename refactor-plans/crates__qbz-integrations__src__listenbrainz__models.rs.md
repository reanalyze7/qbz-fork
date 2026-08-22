# crates/qbz-integrations/src/listenbrainz/models.rs (268 lines)

## Summary
Pure data-model file: serde types for the ListenBrainz API — submission
payloads (`Listen`, `TrackMetadata`, `AdditionalInfo`), auth/status types
(`UserInfo`, `TokenValidationResponse`, `ListenBrainzStatus`), the offline
submission queue row (`QueuedListen`), and the read-side recommendation/
history/playlist/fresh-release types (`CfRecommendation`, `LbListen`,
`LbRecordingMeta`, `LbPlaylistMeta`, `LbPlaylistTrack`, `LbFreshRelease`). No
logic beyond `AdditionalInfo::new`/`with_version`/`Default`.

## Proposed split
Pure by-domain split (no pure/IO/render distinction needed — this whole file
is data definitions): submission-side types vs. read-side types vs. queue/
status types.

- `listenbrainz/models/mod.rs` (~15 lines) — module doc, `pub mod` decls,
  `pub use` re-exports of every type so `listenbrainz::models::X` paths are
  unchanged.
- `listenbrainz/models/submit.rs` (~100 lines) — `ListenType`, `Listen`,
  `TrackMetadata`, `AdditionalInfo` + its `new`/`with_version`/`Default` impl,
  `SubmitListensPayload` (everything needed to build a submission request).
- `listenbrainz/models/status.rs` (~50 lines) — `UserInfo`,
  `TokenValidationResponse`, `ListenBrainzStatus`, `QueuedListen` (auth/
  connection-status/offline-queue types).
- `listenbrainz/models/recommendations.rs` (~50 lines) — `CfRecommendation`,
  `LbListen`, `LbRecordingMeta` (the CF-recommendation + listen-history +
  recording-metadata read types).
- `listenbrainz/models/playlists.rs` (~40 lines) — `LbPlaylistMeta`,
  `LbPlaylistTrack`, `LbFreshRelease` (curated-playlist + fresh-release read
  types).

## Re-export surface
`listenbrainz/models/mod.rs` re-exports every type
(`ListenType`/`Listen`/`TrackMetadata`/`AdditionalInfo`/
`SubmitListensPayload`/`UserInfo`/`TokenValidationResponse`/
`ListenBrainzStatus`/`QueuedListen`/`CfRecommendation`/`LbListen`/
`LbRecordingMeta`/`LbPlaylistMeta`/`LbPlaylistTrack`/`LbFreshRelease`) at
`crate::listenbrainz::models::*` so the ListenBrainz client/service code
(likely `listenbrainz/client.rs` or similar sibling files) that constructs/
deserializes these types keeps its import paths working.

## Coupling / watch out
- Purely additive data types with no cross-references between the proposed
  groups — this is the lowest-risk split in the batch (no shared mutable
  state, no thread-locals, no lifecycle).
- `AdditionalInfo::new()` hardcodes `media_player`/`submission_client` =
  "QBZ" and version "1.0.0"; `with_version` overwrites both version fields —
  keep this logic exactly as-is in `submit.rs`, callers elsewhere in the
  crate likely call `.with_version(actual_app_version)`.
- Several types use `#[serde(rename_all = "camelCase")]` (`ListenBrainzStatus`,
  `QueuedListen`) while most others use the default snake_case — this is
  intentional per-type wire-format matching, not a bug; preserve each type's
  own `#[serde(...)]` attributes verbatim when moving.
- No `#[cfg(test)]` block exists in this file — nothing to preserve test-wise
  beyond compile correctness of the derives.

## Verify after split
- `cargo check -p qbz-integrations` (or full workspace) — confirms the
  serde derives still compile and no field/type got dropped in transit.
- Grep for `listenbrainz::models::` (and any glob `use
  super::models::*`/`use crate::listenbrainz::models::*`) in sibling
  ListenBrainz client files to confirm the public path is unchanged.
- No existing tests to run for this file specifically; rely on the
  `qbz-integrations` crate's broader listenbrainz client tests (if any) still
  passing.
