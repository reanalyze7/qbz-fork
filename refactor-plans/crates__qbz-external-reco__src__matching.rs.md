# crates/qbz-external-reco/src/matching.rs (222 lines)

## Summary
Pure text + ISRC scoring to validate an external recommendation against Qobuz
catalog candidates: ISRC short-circuit, weighted title/artist/album string
similarity plus a duration-closeness bonus, hi-res tiebreak on ties, and
string-normalization helpers (bracket/stop-word stripping) shared with cache
keys.

## Proposed split
Entirely pure functions (no IO) — split by the natural pipeline stage: the
public scoring/selection API vs the string-normalization/similarity internals
vs tests. Only slightly over budget (222 vs 130), so a two-way split plus tests
is enough:

- `matching/mod.rs` (~90 lines) — module doc, weight consts (`TITLE_WEIGHT`,
  `ARTIST_WEIGHT`, `ALBUM_WEIGHT`, `MIN_SCORE`), the `MatchInput` struct,
  `select_best_match` and `score_candidate` (1-102) and `quality_score`
  (188-192) — the public scoring API plus its one small private helper.
- `matching/text.rs` (~90 lines) — `similarity`, `normalize`, `remove_bracketed`,
  `token_overlap` (104-186): the string-normalization/similarity engine, reused
  independently of the Track-scoring logic (e.g. `normalize` is documented as
  also used "for cache keys", a hint it may be called from elsewhere in the
  crate beyond `score_candidate`).
- `matching/tests.rs` (~30 lines) — the `#[cfg(test)] mod tests` block
  (194-222): normalize-strips-brackets-and-stopwords, similarity-exact-and-
  substring, token-overlap-uses-longer-side, normalize-is-stable-for-cache-keys.

## Re-export surface
`matching/mod.rs` re-exports everything currently public:
`pub use text::{similarity, normalize};` alongside its own `pub struct
MatchInput`, `pub fn select_best_match`, `pub fn score_candidate`, `pub const
MIN_SCORE`. Any other module in `qbz-external-reco` (or a sibling crate) that
imports `qbz_external_reco::matching::normalize` or `::similarity` for cache-key
purposes keeps working unchanged.

## Coupling / watch out
- `normalize` is called both by `similarity` (matching.rs original doc: "Clean-
  room port... adapted to score...") AND, per its own doc comment, used
  independently "for cache keys" elsewhere in the crate — grep for
  `matching::normalize` call sites across `qbz-external-reco` before finalizing
  which file it lives in, since it's a cross-cutting utility, not scoring-only.
- `score_candidate`'s ISRC short-circuit (`return 1.0`) happens BEFORE any call
  into `text.rs`'s `similarity` — keep this ordering exactly; it's a
  performance/precision optimization (skip fuzzy matching entirely on ISRC
  agreement), not an accident.
- `quality_score`'s bit_depth*100000.0 + sample_rate formula is a magic-number
  tie-break weighting (bit depth dominates, sample rate is the fractional
  tiebreaker) — keep the comment/context wherever it lands so a future reader
  doesn't "simplify" the formula.

## Verify after split
- `cargo build -p qbz-external-reco` and `cargo build --workspace` (grep for
  external crates depending on `qbz_external_reco::matching::*`).
- `cargo test -p qbz-external-reco matching` — all 4 tests green.
- `cargo clippy -p qbz-external-reco` to confirm no now-unused `pub` visibility
  after the split.
