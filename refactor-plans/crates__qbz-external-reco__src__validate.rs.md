# crates/qbz-external-reco/src/validate.rs (353 lines)

## Summary
Resolves raw recommendation candidates (from Last.fm/ListenBrainz/editorial
sources) to real Qobuz entities: tracks by ISRC/MBID/fuzzy match, artists by
name search, albums by UPC/fuzzy match — each with a shared cache-first
lookup pattern and negative-result caching.

## Proposed split
By entity domain (track/artist/album), following the file's own `// ──
Tracks ──` / `// ── Artists ──` / `// ── Albums ──` banners — a clean,
pre-existing seam. This crate's `src/` is flat (`cache.rs`, `matching.rs`,
`types.rs`, `carousels.rs`, `validate.rs` all declared directly from
`lib.rs`), so convert `validate.rs` into `validate/mod.rs` + 3 siblings,
matching the crate's existing flat-module convention at one level deeper.

- `validate/mod.rs` (~30 lines) — module doc, the shared `Cache<'a>` type
  alias (line 20), `pub use` re-exports of every public fn from the three
  submodules so `crate::validate::validate_track` etc. (used from
  `carousels.rs` and `lib.rs`) keep resolving.
- `validate/track.rs` (~135 lines) — `track_cache_key`, `build_track_reco`,
  `find_by_isrc`, `resolve_track_live`, `validate_track` (lines 70-199). Just
  over budget — if strict, split `resolve_track_live`'s MusicBrainz-ISRC
  fallback branch (120-135) into a small private helper
  `fn resolve_via_musicbrainz_isrcs(...)` to shave a few lines, or accept 135
  as close enough and flag for a human call.
- `validate/artist.rs` (~50 lines) — `validate_artist` (201-249). Already
  well under budget as its own file.
- `validate/album.rs` (~105 lines) — `is_full_album`, `is_slop`,
  `album_if_full`, `album_cache_key`, `build_album_reco`,
  `resolve_album_live`, `validate_album` (22-36, 38-66, 253-353). Note
  `is_full_album`/`is_slop`/`album_if_full` (22-66) are logically
  "pre-filter helpers used by album resolution" — they currently sit at the
  TOP of the file (before the Tracks section) even though they are
  album-specific; move them into `album.rs` alongside the rest of the album
  logic rather than leaving them orphaned in `mod.rs`.

## Re-export surface
`validate/mod.rs` is the public surface: `pub use track::validate_track; pub
use artist::validate_artist; pub use album::{validate_album, is_full_album,
is_slop, build_album_reco};`. The crate's `lib.rs` line `mod validate;` (it's
currently a private, non-`pub`, module per the earlier `grep` — confirm
whether any of these functions are re-exported at the crate root in `lib.rs`
via `pub use validate::...`; if so mirror that exact re-export list) needs no
structural change since `validate/mod.rs` resolves identically to
`validate.rs`.

## Coupling / watch out
- All three submodules share the exact same cache-read/resolve-live/
  cache-write TEMPLATE (lookup by key → on miss resolve → write positive or
  negative back) but with subtly different negative-caching rules per entity
  — `validate_track` has an EXTRA `skip_negative` parameter that
  `validate_artist`/`validate_album` lack (used only for ListenBrainz weekly
  playlists per the doc comment on `validate_track`, lines 152-161). Do not
  "helpfully" unify this into one shared helper during the split — the
  different negative-caching semantics are deliberate (see the comment about
  Weekly Exploration/Jams "vanishing" if a transient miss were cached as a
  7-day negative). Keep each entity's caching logic textually separate even
  though it looks repetitive.
- `RecoCatalog` trait object (`&dyn RecoCatalog`) and `MusicBrainzClient` are
  passed by reference into `track.rs` only — `artist.rs`/`album.rs` only need
  `&dyn RecoCatalog`, not `MusicBrainzClient`; confirm imports are trimmed
  per-file, not blanket-copied.
- `Cache<'a> = Option<&'a Mutex<RecoCache>>` type alias (line 20) is used by
  all three submodules' public functions — define once in `mod.rs`, import
  into each sibling (`use super::Cache;`).
- `MIN_SCORE` (imported from `crate::matching`) is used only in `track.rs`;
  `ALBUM_MIN_SCORE` (a local const, line 22) is used only in `album.rs`
  — do not conflate the two during the split, they are different thresholds
  for different entity types.

## Verify after split
- `cargo build -p qbz-external-reco` and `cargo build --workspace` (the crate
  is consumed by `crates/qbz` for the "Recommendations" tab / external reco
  home rails).
- `cargo test -p qbz-external-reco` — no `#[cfg(test)]` block exists in this
  particular file today (confirm no tests were missed on re-read), but
  `carousels.rs`/`lib.rs` tests that call into `validate_*` must still pass.
- `cargo clippy -p qbz-external-reco`.
- Smoke-test importers: `grep -rn "validate::" crates/qbz-external-reco/src`
  — confirm `carousels.rs`'s calls to `validate_track`/`validate_artist`/
  `validate_album` still compile unchanged.
