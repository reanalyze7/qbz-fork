# crates/qbz-reco/src/suggestions.rs (1137 lines)

## Summary
`SuggestionsEngine` — the largest file in the gap-fill slice. Generates
playlist track suggestions: builds/reuses artist vectors, ranks related
artists by summed relationship weight (not cosine), searches Qobuz for
tracks, validates artists exist in Qobuz with compatible genre, dedupes/
shuffles/truncates the result pool.

## Proposed split
By pipeline stage / concern — this file has very clear phase boundaries
already marked by numbered step comments (1-7) inside `generate_suggestions`:

- `suggestions/mod.rs` (~90 lines) — `SuggestionConfig`, `SuggestedTrack`,
  `SuggestionResult`, `SuggestionsEngine` struct + `new()`, `pub use` of
  submodules.
- `suggestions/generate.rs` (~230 lines) — the `generate_suggestions` method
  itself (steps 1-7: ensure vectors, compute playlist vector, find related
  artists, search tracks, Qobuz-similar fallback, dedupe/shuffle/truncate).
  This is the biggest chunk and still exceeds 130 — further split by step:
  - `suggestions/generate/core.rs` (~130 lines) — steps 1-3 (ensure vectors,
    compute playlist vector, find related artists) + final assembly
    (dedupe/shuffle/truncate/return).
  - `suggestions/generate/search_playlist_artists.rs` (~55 lines) — step 4a.
  - `suggestions/generate/search_related_artists.rs` (~55 lines) — step 4b.
  - `suggestions/generate/qobuz_similar_fallback.rs` (~100 lines) — step 4c.
- `suggestions/playlist_vector.rs` (~20 lines) — `compute_playlist_vector`.
- `suggestions/track_search.rs` (~200 lines) — `search_artist_tracks`,
  `search_artist_tracks_with_limit`, `search_artist_tracks_by_qobuz_id`.
- `suggestions/validate_artist.rs` (~90 lines) — `validate_qobuz_artist`.
- `suggestions/genre_filter.rs` (~155 lines) — `has_incompatible_genre` +
  its two large `const` keyword-blocklist arrays
  (`INCOMPATIBLE_GENRES`, `INCOMPATIBLE_TITLE_KEYWORDS`); still over 130 —
  move the const arrays to `suggestions/genre_filter/blocklist.rs` (~110
  lines of just the two arrays) and keep the ~45-line function in
  `genre_filter.rs`.
- `suggestions/track_convert.rs` (~90 lines) — `track_to_suggested`,
  `track_to_suggested_with_qobuz_id` (near-duplicate Track→SuggestedTrack
  mappers — consider merging into one fn with an `Option<u64>` mbid param
  during a real split, but that's a behavior change so just flag it here).
- `suggestions/reason.rs` (~15 lines) — `generate_reason`.
- `suggestions/name_match.rs` (~90 lines) — free fns `normalize_name`,
  `names_similar` (pure string logic, no `self`, easiest to unit test in
  isolation).
- `suggestions/mbids.rs` (~20 lines) — free fn `extract_artist_mbids`.
- `suggestions/tests.rs` (~35 lines) — existing test module.

## Re-export surface
`suggestions/mod.rs` stays the `mod suggestions;` target. Public items used
by callers outside the crate: `SuggestionConfig`, `SuggestedTrack`,
`SuggestionResult`, `SuggestionsEngine`, `extract_artist_mbids` — all must
stay reachable at `crate::suggestions::X`. `SuggestionsEngine`'s methods are
split across many files as additional `impl SuggestionsEngine` blocks
(`use super::SuggestionsEngine;` in each), which is transparent to callers.

## Coupling / watch out
- This is the single trickiest split in the slice: `generate_suggestions` is
  one long async fn with heavy internal state (`all_tracks`, `source_artists`,
  `exclude_track_ids`) threaded through steps 1-7 sequentially — splitting by
  "step" only works if each step becomes its own private method taking/
  returning that shared state explicitly (not a trivial mechanical split;
  flag for the real split PR to design the intermediate struct/tuple shape).
- Locking pattern: `self.store.lock().await` and `self.qobuz_client.read().await`
  guards are scoped with `{ }` blocks and dropped before subsequent `.await`s
  — same discipline as `builder.rs`; preserve when methods move to new files.
- `track_to_suggested` vs `track_to_suggested_with_qobuz_id` are ~90% duplicated
  — do not silently merge them during a mechanical split (that's a logic
  change); just move both verbatim and note the duplication for a follow-up.
- The two `const` blocklists in `has_incompatible_genre` are large but
  logically single data tables — don't split them further than one file.
- `search_artist_tracks_with_limit` calls `validate_qobuz_artist` which calls
  `has_incompatible_genre` — three-file call chain
  (`track_search.rs` → `validate_artist.rs` → `genre_filter.rs`), all need
  `use super::SuggestionsEngine;` plus `use qbz_qobuz::QobuzClient;`.

## Verify after split
- `cargo test -p qbz-reco suggestions::` — both existing tests
  (`test_extract_artist_mbids`, `test_suggestion_config_default`) green.
- `cargo check -p qbz-reco` and check `Send` bounds still hold for the
  spawned suggestions future (same concern as builder.rs).
- Manually exercise "generate playlist suggestions" in the app (or add an
  integration test) since genre-filter/name-match logic has no direct unit
  coverage today — a bad split here would be silent at compile time.
