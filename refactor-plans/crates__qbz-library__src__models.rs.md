# crates/qbz-library/src/models.rs (298 lines)

## Summary
Data models for the local library: `AudioFormat` enum, `LocalTrack` (+
`Default`), `PlaylistLocalTrack`, album aggregation types
(`AlbumsMetadataPage`, `LocalAlbum`), `LocalArtist`, scan-progress types
(`ScanProgress`, `ScanStatus`, `ScanError`), `AudioProperties`,
`AlbumSettings` (+ constructor), `FolderTreeEntry`, `ArtistImageInfo`. Pure
data, no I/O, no `#[cfg(test)]` module.

## Proposed split
By domain — the file has no section banners but the structs group cleanly:

- `models/mod.rs` (~15 lines) — module doc + `pub use` re-export of every
  submodule's public items, so `qbz_library::models::LocalTrack` etc. (or
  wherever this re-exports from `qbz_library`'s crate root) stay unchanged.
- `models/track.rs` (~90 lines) — lines 5-134: `AudioFormat` (+ `Default`,
  `Display`), `LocalTrack` (+ `Default`), `PlaylistLocalTrack`.
- `models/album.rs` (~90 lines) — lines 136-178: `AlbumsMetadataPage`,
  `LocalAlbum`, `default_source()` helper, plus `LocalArtist` (lines
  180-186) since it's a small closely-related aggregation type — if this
  pushes past budget move `LocalArtist` into its own tiny
  `models/artist.rs`.
- `models/scan.rs` (~50 lines) — lines 188-234: `ScanProgress` (+
  `Default`), `ScanStatus`, `ScanError`, `AudioProperties`.
- `models/album_settings.rs` (~25 lines) — lines 236-259: `AlbumSettings` +
  its `new()` constructor.
- `models/folder_tree.rs` (~30 lines) — lines 261-287: `FolderTreeEntry`
  (with its detailed doc comment on the `kind`/`path`/`segment`/
  `track_count_under`/`artwork` field semantics — preserve verbatim).
- `models/artist_image.rs` (~10 lines) — lines 289-297: `ArtistImageInfo`.

## Re-export surface
`models/mod.rs` re-exports every submodule with `pub use track::*; pub use
album::*; pub use scan::*; pub use album_settings::*; pub use
folder_tree::*; pub use artist_image::*;` so every existing caller doing
`qbz_library::models::LocalTrack`, `...::LocalAlbum`, `...::ScanProgress`
etc. (or the flattened `qbz_library::LocalTrack` if the crate root already
does `pub use models::*`) keeps resolving identically. Check
`crates/qbz-library/src/lib.rs` for the existing re-export pattern before
finalizing submodule names.

## Coupling / watch out
- `PlaylistLocalTrack` wraps `LocalTrack` via `#[serde(flatten)]` — both
  must be visible to each other; keep them in the same file
  (`models/track.rs`) as planned, or if split further ensure
  `playlist_local_track.rs` does `use super::track::LocalTrack;`.
- `LocalAlbum.format: AudioFormat` and `LocalTrack.format: AudioFormat`
  both reference the enum defined in `models/track.rs` — `album.rs` needs
  `use super::track::AudioFormat;`.
- `AlbumsMetadataPage.albums: Vec<LocalAlbum>` — intra-file within
  `album.rs`, no cross-module concern if kept together as planned.
- None of these types have complex serde attributes beyond
  `#[serde(default)]`/`#[serde(flatten)]`/`#[serde(tag = ..., rename_all =
  ...)]` on `FolderTreeEntry` — straightforward to relocate verbatim.
- No existing test coverage in this file — the split's correctness rests
  entirely on compilation succeeding plus whatever integration tests exist
  elsewhere in `qbz-library` (e.g. database/scanner tests) that construct
  these types.

## Verify after split
- `cargo check -p qbz-library` and `cargo test -p qbz-library` (exercises
  any scanner/database tests elsewhere in the crate that construct
  `LocalTrack`/`LocalAlbum`/etc., indirectly validating the models split).
- `cargo check` across dependent crates (`qbz`, `qbz-app`, `qbzd` all likely
  consume `qbz_library::models::*` for the local-library UI/CLI surfaces) —
  grep for `qbz_library::models::` or `qbz_library::LocalTrack` etc. across
  the workspace to confirm every call site still resolves post-split.
- Smoke-test the local library scan + Folders tab in the running app (or
  `qbzd`'s library-related CLI surfaces, if any) to confirm serialization
  (JSON shape for `FolderTreeEntry`'s `kind`-tagged enum in particular)
  is byte-identical to before the split.
