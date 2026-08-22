# crates/qbz-models/src/types.rs (1281 lines)

## Summary
Monolithic "all shared API types" crate root: quality/stream types, image
sets, core media types (Track/Album/Artist/Playlist), label/genre metadata,
search/favorites/discover response types, and the `/artist/page` response
type family, plus two unit tests on `UserSession`.

## Proposed split
By domain, since this is pure data (serde structs/enums + a handful of small
pure helper methods) with almost no cross-cutting logic — mirrors the file's
own `// ============ Section ============` banner comments:

- `types/mod.rs` (~20 lines) — module doc + `pub use` re-export of every
  submodule's public items, so `qbz_models::types::Track` etc. keep working.
- `types/quality.rs` (~130 lines) — lines 12-116: `TrackToAnalyse`, `Quality`
  (+ impls), `QualityLimit`.
- `types/session.rs` (~25 lines) — lines 118-138: `UserSession`. Also owns
  the two `#[cfg(test)]` tests (lines 1239-1280) since they only exercise
  `UserSession`.
- `types/stream.rs` (~130 lines) — lines 140-217: `StreamUrl`,
  `StreamRestriction`, `StreamQualityInfo` (+ impls incl. `from_raw`).
- `types/external_stream.rs` (~100 lines) — lines 219-296: `AudioParams`,
  `probe_streaminfo`, `AssetOrigin`, `ExternalStreamAsset` (+ custom `Debug`).
- `types/cmaf.rs` (~40 lines) — lines 286-329 minus the external-stream bits
  already carved out: `SessionStartResponse`, `TrackFileUrl`.
- `types/image.rs` (~35 lines) — lines 330-363: `ImageSet` (+ `best`/`smallest`).
- `types/media.rs` (~140 lines) — lines 365-530ish: `Track`, `AlbumSummary`,
  `Album`, `AlbumArtist`, `Goody`. Still likely >130 with `Album`'s many
  fields — if so split further into `types/media/track.rs` (Track +
  AlbumSummary) and `types/media/album.rs` (Album, AlbumArtist, Goody).
- `types/playlist.rs` (~130 lines) — `TracksContainer`, `Artist`,
  `ArtistBiography`, `ArtistAlbums`, `Playlist`, `PlaylistOwner`,
  `PlaylistGenre`, `PlaylistWithTrackIds`, `PlaylistDuplicateResult`. Likely
  needs splitting into `types/artist.rs` (Artist family) and
  `types/playlist.rs` (Playlist family) given the combined size (~230 lines
  currently, lines 533-655).
- `types/label.rs` (~130 lines) — `Label`, `LabelPageData`,
  `LabelPageContainer`, `LabelPageGenericList`, `LabelExploreResponse`,
  `LabelListPage<T>`, `LabelStoryResponse`, `LabelGetListResponse` (lines
  659-767). Genre types (`Genre`, `GenreInfo`, `GenreListResponse`,
  `GenreListContainer`, lines 769-802) can stay here or move to a small
  `types/genre.rs` if label.rs is still oversized.
- `types/search.rs` (~130 lines) — `SearchResults`, `SearchResultsPage<T>`,
  `AlbumSuggestResponse`, `MostPopularItem`, `SearchAllResults`, `Favorites`
  (lines 804-869).
- `types/discover.rs` (~140 lines, likely split into
  `discover/response.rs` + `discover/album.rs`) — `DiscoverResponse`,
  `DiscoverContainers`, `DiscoverContainer<T>`, `DiscoverData<T>`,
  `DiscoverPlaylist`, `DiscoverPlaylistImage`, `PlaylistTag`,
  `RawPlaylistTag`, `PlaylistTagsResponse`, `DiscoverPlaylistsResponse`,
  `DiscoverAlbum`, `DiscoverAlbumImage`, `DiscoverArtist`,
  `DiscoverAlbumDates`, `DiscoverAudioInfo` (lines 871-1005).
- `types/artist_page.rs` (~230 lines, needs splitting into
  `artist_page/mod.rs` + `artist_page/release.rs` + `artist_page/track.rs`)
  — the entire `/artist/page` and `/artist/story` type family (lines
  1007-1237): `PageArtistResponse` through `ArtistStoryAuthor`, plus
  `ReleasesGridResponse`.

## Re-export surface
`types/mod.rs` (or, if this crate's `lib.rs` currently does `pub mod types;`
directly re-exporting `pub use types::*`, keep that untouched) re-exports
every submodule with `pub use quality::*; pub use session::*; ...` etc. so
existing callers doing `qbz_models::Track` / `qbz_models::types::Track`
continue to resolve identically. Check `crates/qbz-models/src/lib.rs` first
to see whether it does `pub use types::*` — if so, ALL type names stay at
`qbz_models::TypeName` regardless of the internal `types/` split, which
should make this split very low-risk for external callers.

## Coupling / watch out
- Cross-references are everywhere in this file: `Track.performer: Option<Artist>`,
  `Track.album: Option<AlbumSummary>`, `Album.label: Option<Label>`,
  `Album.genre: Option<Genre>`, `Album.audio_info: Option<DiscoverAudioInfo>`,
  `Album.dates: Option<DiscoverAlbumDates>`, `AlbumSummary.label`/`.genre`,
  `Artist.albums: Option<ArtistAlbums>`, `ArtistAlbums.items: Vec<Album>`,
  `Playlist.tracks: Option<TracksContainer>`, `TracksContainer.items: Vec<Track>`.
  Splitting by domain module means many submodules need `use super::{Track,
  Album, Artist, Genre, Label, ImageSet, ...}` — get every cross-module `use`
  right or it won't compile. Recommend doing the split with the compiler as
  a guide (fix one `use` error at a time) rather than trying to fully
  pre-plan every import.
- `DiscoverAudioInfo` and `DiscoverAlbumDates` are referenced from BOTH the
  discover types (their origin) AND `Album` (the "V2 nested" fields) — these
  need to live in a module both `media.rs`/`album.rs` and `discover.rs` can
  import from without a cycle; keep them in `discover.rs` and have
  `album.rs` `use super::discover::{DiscoverAudioInfo, DiscoverAlbumDates};`.
- `PageArtistName`, `PageArtistReleaseArtist`, `ImageSet`, `Label`, `Genre`
  are reused across the artist_page family and the top-level media types —
  same cross-module import care needed.
- Generic types (`LabelListPage<T>`, `SearchResultsPage<T>`,
  `DiscoverContainer<T>`, `DiscoverData<T>`) have no special coupling beyond
  needing `Serialize`/`Deserialize`/`Default` bounds already in scope.
- The two `#[cfg(test)]` tests only need `UserSession` in scope — trivial to
  relocate.

## Verify after split
- `cargo check -p qbz-models` and `cargo test -p qbz-models` (runs the two
  `UserSession` tests).
- `cargo check` (or `cargo build`) across the whole workspace, since this is
  a foundational types crate consumed by nearly every other crate (`qbz`,
  `qbz-app`, `qbzd`, `qbz-library`, `qbz-qobuz`, etc.) — any broken
  re-export surfaces immediately as a compile error somewhere downstream.
- Diff `cargo doc` output for `qbz_models` before/after (public path changes
  would show up as moved/missing items) if available, or just grep the
  workspace for `qbz_models::types::` to confirm no caller depended on the
  literal `types::SubmoduleName::` path rather than the flattened re-export.
