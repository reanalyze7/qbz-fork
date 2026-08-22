# crates/qbz-library/src/thumbnails.rs (168 lines)

## Summary
Album-artwork thumbnail generation and cache management: resolve the
thumbnails directory, hash a source path or cache key into a filename,
generate/resize/save a thumbnail from a file path or raw bytes, and
clear/size the cache — no tests currently present in this file.

## Proposed split
Modest overage (168 vs 130); split by concern: directory/path resolution
vs. generation vs. cache maintenance.

- `thumbnails/mod.rs` (~20 lines) — module doc, `pub mod` declarations,
  `pub use` re-exports of every public fn so `crate::thumbnails::X` paths
  (i.e. `qbz_library::thumbnails::X`) are unchanged.
- `thumbnails/paths.rs` (~45 lines) — `THUMBNAIL_SIZE`, `get_thumbnails_
  dir`, `get_thumbnail_filename`, `get_thumbnail_path`, `thumbnail_exists`.
- `thumbnails/generate.rs` (~75 lines) — `generate_thumbnail`,
  `generate_thumbnail_from_bytes`, `get_or_generate_thumbnail`.
- `thumbnails/cache.rs` (~40 lines) — `clear_thumbnails`, `get_cache_
  size`.

## Re-export surface
`thumbnails/mod.rs` re-exports `get_thumbnails_dir`, `get_thumbnail_path`,
`thumbnail_exists`, `generate_thumbnail`, `generate_thumbnail_from_bytes`,
`get_or_generate_thumbnail`, `clear_thumbnails`, `get_cache_size` at
`crate::thumbnails::*` (`qbz_library::thumbnails::*`) — consumed wherever
artwork is displayed/imported in the library scan and command layers;
grep for `thumbnails::` call sites in `qbz-library` and any frontend
crate before finalizing.

## Coupling / watch out
- `get_thumbnail_filename` (path-based, DefaultHasher of the path string)
  and the inline hashing block inside `generate_thumbnail_from_bytes`
  (cache-key-based) are two SEPARATE hashing schemes producing filenames
  in the same shared `thumbnails_dir` — they don't collide today only
  because path-strings and cache-keys happen not to hash-collide in
  practice, but they are genuinely two different derivations. Consider
  factoring a single `hash_to_filename(input: &str) -> String` helper in
  `paths.rs` that both `get_thumbnail_filename` and `generate_thumbnail_
  from_bytes` call, rather than leaving the byte-hashing logic duplicated
  inline inside `generate.rs` — a good opportunistic cleanup during this
  split, not just a relocation.
- `generate_thumbnail` and `generate_thumbnail_from_bytes` both duplicate
  the "check exists → decode → resize Lanczos3 → save JPEG" sequence
  (differing only in decode source: `ImageReader::open` a path vs.
  `ImageReader::new(Cursor)` + `with_guessed_format`) — keep both in
  `generate.rs` together so the duplication stays visible/fixable in one
  place; a shared private helper taking the already-decoded
  `DynamicImage` would remove the duplication if the agent doing the real
  split wants to go further than the plan strictly requires.
- `get_or_generate_thumbnail` is a thin wrapper calling both `paths::get_
  thumbnail_path` and `generate::generate_thumbnail` — it's the most
  natural candidate for `thumbnails/mod.rs` itself (a facade fn) rather
  than living in either `paths.rs` or `generate.rs`; either placement
  works, just keep it visible from `mod.rs`.
- No `#[cfg(test)]` block exists in this file today — per the project's
  "tests at each change" rule, add at least a couple of unit tests
  (`get_thumbnail_filename` determinism/uniqueness, `clear_thumbnails`
  leaves the dir present-but-empty) as part of the real split rather than
  moving code untested.

## Verify after split
- `cargo test -p qbz-library thumbnails::` (once tests are added per the
  note above) green.
- `cargo check -p qbz-library` (and any frontend crate using
  `qbz_library::thumbnails::*`) to confirm import paths still resolve.
- Manual smoke-test: import/scan a local library with album art, confirm
  thumbnails generate and display, delete the thumbnails cache via
  whatever UI exposes `clear_thumbnails`, confirm it regenerates on next
  view.
