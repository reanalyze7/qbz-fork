# crates/qbz-integrations/src/musicbrainz/models.rs (653 lines)

## Summary
Plain serde data models for the MusicBrainz integration: raw API response DTOs
(recording/artist/release search & lookup, relations, areas), a set of "resolved"
output types used for caching, and higher-level domain types for location discovery,
tag-based discovery, and musician/appearance matching. No behavior beyond a few
small enums' `from_score`/`level`/`Default` impls.

## Proposed split
By domain group (this is pure data — the split follows the file's own section
comments, which already delineate the boundaries cleanly).

- `musicbrainz/models/mod.rs` (~15 lines) — module doc + `pub use *` re-exports from
  every submodule below, so `qbz_integrations::musicbrainz::models::X` (or however
  it's currently re-exported at `musicbrainz::X`) is unchanged.
- `musicbrainz/models/confidence.rs` (~60 lines) — `MatchConfidence` (+`from_score`),
  `ArtistType` (+`Default`, `From<Option<&str>>`), `MusicianConfidence` (+`level`,
  `Default`) — the small shared enums used across the other model groups.
- `musicbrainz/models/recording.rs` (~90 lines) — `RecordingSearchResponse`,
  `RecordingResult`, `RecordingLookupResponse`, `ArtistCredit`, `ArtistRef`,
  `ReleaseRef`, `ReleaseGroupRef` (the recording/search-result cluster).
- `musicbrainz/models/artist.rs` (~110 lines) — `ArtistSearchResponse`, `ArtistResult`,
  `Alias`, `Area`, `Tag`, `LifeSpan`, `Relation`, `ArtistFullResponse`,
  `ArtistBrowseResponse` (the artist-lookup/search cluster).
- `musicbrainz/models/release.rs` (~90 lines) — `ReleaseSearchResponse`,
  `ReleaseResult`, `ReleaseSearchMedium`, `LabelInfo`, `LabelRef`,
  `ReleaseFullResponse`, `Medium`, `MediumTrack`, `TrackRecording` (the release-lookup
  cluster).
- `musicbrainz/models/area.rs` (~70 lines) — `AreaSearchResponse`, `AreaResult`,
  `AreaDetailResponse`, `AreaRelation`, `AreaRelTarget`.
- `musicbrainz/models/relationships.rs` (~50 lines) — `RelatedArtist`, `Period`,
  `ArtistRelationships` (+ `empty`/`is_empty`).
- `musicbrainz/models/resolved.rs` (~40 lines) — `ResolvedArtist`, `ResolvedTrack`,
  `ResolvedRelease` (the "for caching/output" resolved types).
- `musicbrainz/models/discovery.rs` (~110 lines) — `LocationPrecision`,
  `ArtistLocation`, `AffinitySeeds`, `ArtistMetadata`, `LocationCandidate`,
  `LocationDiscoveryResponse`, `DiscoveryArtist`, `DiscoveryResponse` (location +
  tag-based discovery).
- `musicbrainz/models/musician.rs` (~60 lines) — `ResolvedMusician` (+`empty`),
  `AlbumAppearance`, `MusicianAppearances` (the musician-matching cluster).

## Re-export surface
`musicbrainz/models/mod.rs` re-exports every public type with `pub use`, so any
existing `use qbz_integrations::musicbrainz::{ArtistType, ArtistMetadata, ...}` or
`crate::musicbrainz::models::X` path elsewhere in the workspace (notably
`crates/qbz/src/artist.rs`, which imports `qbz_integrations::musicbrainz::{ArtistType,
LocationPrecision}` directly) keeps compiling unchanged — check the module's parent
(`musicbrainz/mod.rs`) already re-exports `models::*` at the `musicbrainz::` level; if
so nothing there needs to change.

## Coupling / watch out
- `ArtistType` (confidence.rs) is used inside `ArtistMetadata` (discovery.rs) and
  `ResolvedArtist` (resolved.rs) — cross-module `use super::confidence::ArtistType;`
  needed in both.
- `LifeSpan` (artist.rs) is reused by `ArtistMetadata` (discovery.rs) — cross-import.
  `Tag` (artist.rs) is reused by `AffinitySeeds`... actually `AffinitySeeds` doesn't
  reference `Tag` directly, but the resolver code elsewhere likely maps `Vec<Tag>` to
  `AffinitySeeds` — check call sites in `musicbrainz/mod.rs` or similar for functions
  bridging artist.rs types to discovery.rs types.
- `ArtistRef` and `ArtistCredit` (recording.rs) are also referenced from
  `release.rs`'s `ReleaseResult`/`ReleaseFullResponse`/`TrackRecording`
  (`artist-credit` field) — cross-module import needed there.
- `ReleaseGroupRef` (recording.rs) is reused by `release.rs`'s `ReleaseResult`.
- `Area` (artist.rs) is used for `ArtistResult.area`/`begin_area` — stays local to
  artist.rs, fine.
- None of these types have inherent behavior beyond the few enums in confidence.rs,
  so the split itself is low-risk; the main work is untangling the cross-references
  listed above with correct `use` paths.

## Verify after split
- `cargo check -p qbz-integrations` (this crate likely has serde round-trip tests
  elsewhere in the `musicbrainz` module — run `cargo test -p qbz-integrations
  musicbrainz` too).
- `cargo check` across the workspace (or at least `-p qbz`, which directly imports
  `qbz_integrations::musicbrainz::{ArtistType, LocationPrecision, ArtistMetadata,
  ArtistRelationships, RelatedArtist}` per `crates/qbz/src/artist.rs`) to catch any
  broken import path.
