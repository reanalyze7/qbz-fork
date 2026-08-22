# crates/qbz-playlist-import/src/match_qobuz.rs (553 lines)

## Summary
Matches imported playlist tracks (from Spotify/Apple/Deezer) against the
Qobuz catalog: concurrent search (bounded by `CONCURRENCY`), a weighted
title/artist/album fuzzy-similarity scorer with an ISRC short-circuit and a
hi-res quality tiebreak, plus string-normalization helpers (bracket
stripping, stop-word removal, token overlap).

## Proposed split
By domain — orchestration/concurrency vs. pure scoring vs. pure string
normalization — the file already reads top-to-bottom in exactly this order:

- `match_qobuz/mod.rs` (~10 lines) — module doc, `pub use
  matcher::match_tracks;` (the only public item — `select_best_match`,
  `score_candidate`, etc. are private implementation detail).
- `match_qobuz/matcher.rs` (~110 lines) — the async `match_tracks()`
  function: buffered-concurrent search over tracks, per-track match-entry
  construction, progress event emission, and result reassembly in original
  order via the indexed `results` vector.
- `match_qobuz/scoring.rs` (~110 lines) — `select_best_match`,
  `score_candidate`, `quality_score` — the weighted scoring + ISRC
  short-circuit + hi-res tiebreak logic (pure, given `ImportTrack` +
  `&[Track]`/`&Track`).
- `match_qobuz/normalize.rs` (~100 lines) — `similarity`, `normalize`,
  `remove_bracketed`, `token_overlap` — pure string utilities with no
  dependency on `Track`/`ImportTrack` types at all beyond `&str`.
- `match_qobuz/tests.rs` (~250 lines) — the entire existing `#[cfg(test)]
  mod tests` block. Even split three ways among the modules it tests
  (normalize/remove_bracketed/token_overlap → normalize.rs tests;
  similarity → normalize.rs tests; score_candidate/select_best_match/
  quality_score → scoring.rs tests) OR kept as one file with `use
  super::super::{scoring::*, normalize::*};` — prefer splitting the test
  module to mirror the three source files (each under its own
  `#[cfg(test)] mod tests` at the bottom of `scoring.rs` and
  `normalize.rs`) since each test group already only exercises one file's
  functions and none currently cross-test scoring+normalize together.

## Re-export surface
`match_qobuz/mod.rs` re-exports only `match_tracks` — that's the sole
symbol `importer.rs` (formerly `crate::match_qobuz::match_tracks`) imports.
Everything else (`select_best_match`, `score_candidate`, `similarity`,
`normalize`, etc.) stays module-private/`pub(crate)` at most, since no
external caller uses them — check with `grep -rn
"match_qobuz::(select_best_match|score_candidate|similarity|normalize|remove_bracketed|token_overlap|quality_score)"
crates` before dropping their pub-ness, in case something in `qbzd` or a
test elsewhere reaches in directly.

## Coupling / watch out
- `matcher.rs` calls `select_best_match` (from `scoring.rs`) — needs
  `use super::scoring::select_best_match;`.
- `scoring.rs`'s `score_candidate` calls `similarity` (from
  `normalize.rs`) three times — needs `use super::normalize::similarity;`.
- The weight constants (`TITLE_WEIGHT`, `ARTIST_WEIGHT`, `ALBUM_WEIGHT`,
  `MIN_SCORE`) belong in `scoring.rs` (used by `score_candidate`); `MIN_SCORE`
  is ALSO referenced by `matcher.rs`'s match-acceptance check (`score >=
  MIN_SCORE`) — export it as `pub(crate)` from `scoring.rs` or hoist it to
  `mod.rs` since it's a shared threshold between phases.
  `SEARCH_LIMIT` and `CONCURRENCY` stay in `matcher.rs` (only used there).
- The comment at test `select_best_match_score_below_min_score_threshold`
  is explicit about non-obvious behavior (select_best_match does NOT gate
  on MIN_SCORE, the caller does) — this nuance must not get lost; keep the
  full comment when moving that test.
- `normalize()`'s stop-word list and the CJK/non-ASCII-degrades-to-spaces
  behavior are both explicitly locked by tests — do not "fix" or refactor
  the behavior while moving code, only relocate it.

## Verify after split
- `cargo test -p qbz-playlist-import match_qobuz` (or however module paths
  resolve post-split) — all ~19 existing tests must stay green, including
  the ISRC case-insensitivity test, duration-bonus-tier test, and hi-res
  tiebreak test.
- `cargo check -p qbz-playlist-import` to confirm `importer.rs`'s
  `use crate::match_qobuz::match_tracks;` still resolves.
