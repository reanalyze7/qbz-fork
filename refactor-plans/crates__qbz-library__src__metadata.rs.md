# crates/qbz-library/src/metadata.rs (1321 lines)

## 1. Summary

`MetadataExtractor` — a single struct (all associated functions, no
instance state) that turns an on-disk audio file into a `LocalTrack`:
cross-tag lofty reads with per-key fallback across all tags, filename
track-number inference, disc-folder/encoding-folder detection and
album-root-dir resolution, artist/album inference from folder structure,
DSD extraction via `qbz-dsd`, plus artwork extraction/caching/scoring —
and a large `#[cfg(test)]` module covering all of the above.

## 2. Proposed module split

| New file | Owns | ~lines |
|---|---|---|
| `metadata/mod.rs` | `pub struct MetadataExtractor;` decl + `impl` blocks are spread across submodules via `impl MetadataExtractor` in each file (Rust allows splitting impls across files in the same module tree only via separate `impl` blocks, which is fine here); re-exports; module docs | ~20 |
| `metadata/tags.rs` | Cross-tag reading core: `tags_primary_first`, `string_across_tags`, `string_from_tags`, `track_across_tags`, `track_from_tags`, `disk_across_tags`, `disk_from_tags`, `year_across_tags`, `year_from_tags`, `normalize_field` | ~110 |
| `metadata/naming.rs` | Pure string/name parsing: `strip_year_suffix`, `strip_disc_suffix`, `is_disc_marker`, `is_disc_designator`, `is_disc_folder`, `disc_number_from_name`, `infer_track_number_from_filename` | ~230 |
| `metadata/folder_layout.rs` | Folder-structure inference: `is_encoding_folder`, `album_root_dir`, `infer_artist_album`, `infer_disc_number`, `album_group_info` | ~130 |
| `metadata/extract.rs` | `extract`, `extract_with_roots`, `extract_dsd`, `extract_properties`, `detect_format` — the main `LocalTrack`-building entry points | ~290 |
| `metadata/artwork.rs` | `extract_artwork`, `cache_artwork_file`, `find_folder_artwork`, `normalize_artwork_key`, `is_supported_artwork_ext`, `artwork_score` | ~220 |
| `metadata/tests.rs` | The entire `#[cfg(test)] mod tests` block, unchanged (verbatim move) | ~350 |

This is a by-domain split (tag-reading / name-parsing / folder-layout /
top-level-extract / artwork), not pure/IO/render — `MetadataExtractor` is
inherently a mix of pure parsing helpers and filesystem-touching
extraction; the split groups by concern rather than by purity, matching
the README's "by-domain" fallback.

## 3. Re-export / public API surface

`metadata/mod.rs` declares the submodules and re-exports the struct:

```rust
mod artwork;
mod extract;
mod folder_layout;
mod naming;
mod tags;
#[cfg(test)]
mod tests;

pub struct MetadataExtractor;
```

Since all the real logic lives in `impl MetadataExtractor { ... }` blocks
inside each submodule file (Rust permits multiple `impl` blocks for the
same type across files as long as they're all in the same crate), no
re-export shimming is needed beyond the single struct declaration staying
in `mod.rs`. Every external caller doing
`qbz_library::metadata::MetadataExtractor::extract(...)` etc. keeps
working unchanged — the type's path doesn't move.

## 4. Tricky coupling/shared state to watch out for

- All the "public API" methods (`extract`, `extract_with_roots`,
  `extract_properties`, `detect_format`, `extract_artwork`,
  `cache_artwork_file`, `find_folder_artwork`,
  `infer_track_number_from_filename`, `infer_disc_number`,
  `album_group_info`) are called cross-module inside this same file
  today (e.g. `extract` calls `Self::infer_artist_album`,
  `Self::album_group_info`, `Self::detect_format`) — after the split
  these become calls into sibling `impl` blocks in *other* files, which
  works fine in Rust but means every `Self::foo()` call still resolves
  correctly only because they're all inherent methods on the same type;
  double-check no method was accidentally made module-private (`fn` vs
  `pub(crate) fn`) in a way that blocks cross-file access — inherent impl
  methods have no visibility issue across files in the same crate, so
  this should be a non-issue, but worth a build check.
- `extract_dsd` depends on `infer_artist_album`, `infer_disc_number`, and
  `album_group_info` (folder_layout.rs) plus `normalize_field` (tags.rs)
  — that's the one function crossing three of the new module
  boundaries; keep its imports (`use super::folder_layout::...` style,
  or just call `Self::...` since they're all still `MetadataExtractor`
  methods) correct.
- The extensive doc comments on cross-tag fallback (`#447`/`#507`) are
  load-bearing context — they explain WHY the multi-tag iteration exists
  and must move with `tags.rs`, not get trimmed.
- Test module references private-ish helpers directly (`string_from_tags`,
  `track_from_tags`, `disk_from_tags`, `year_from_tags`,
  `is_disc_folder`, `album_root_dir`, `infer_artist_album`,
  `is_encoding_folder`) via `super::*` — after the split, `tests.rs`'s
  `use super::*;` must resolve to `mod.rs`'s re-exported surface, or the
  tests need `use super::tags::*; use super::folder_layout::*;` etc.
  Given these are inherent methods, `use super::*` in tests.rs actually
  just needs `MetadataExtractor` in scope, so a single `use
  super::MetadataExtractor;` (plus whatever else tests use directly,
  like `AudioFormat`, `Path`, `PathBuf`) should suffice.

## 5. What to verify after the real split

- `cargo build -p qbz-library` and `cargo test -p qbz-library metadata::`
  — all ~20 tests green (cross-tag fallback tests, disc-folder tests,
  album-root-dir tests, track-number-inference tests, format-detection
  test).
- Grep the workspace for `qbz_library::metadata::` /
  `MetadataExtractor::` usages outside this crate (`qbz`, `qbz-app`
  likely call `extract`/`extract_with_roots`/`extract_properties` during
  library scanning) to confirm import paths are unaffected (they should
  be, since the type's crate path doesn't change).
- Smoke-test an actual library scan/rescan on a small folder with mixed
  disc/encoding subfolders and a DSD file, confirming album/artist/track
  numbers and cover art still resolve identically to before the split.
