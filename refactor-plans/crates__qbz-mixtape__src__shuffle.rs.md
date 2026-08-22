# crates/qbz-mixtape/src/shuffle.rs (761 lines)

DJ-mix sampling: title/artist normalization, similarity-based dedup
(union-find + token-set ratio), and album-capped hybrid sampling. Pure
functions, no Tauri types.

## Proposed split

- `shuffle/mod.rs` (~30 lines) — re-export surface + consts
  (`SIMILARITY_THRESHOLD`, `ALBUM_CAP_PCT`, `ALBUM_CAP_MIN`).
- `shuffle/normalize.rs` (~90 lines) — `normalize_title`, `normalize_artist`,
  `strip_diacritics`, `remove_brackets`, `strip_dash_suffix`, `strip_feat`,
  `strip_punctuation`, `collapse_whitespace`.
- `shuffle/similarity.rs` (~90 lines) — `token_set_ratio`,
  `join_with_intersection`, `build_similarity_groups`, `uf_find`,
  `uf_union` (union-find grouping).
- `shuffle/dedup.rs` (~50 lines) — `unique_track_count`,
  `dedup_by_similarity`.
- `shuffle/sample.rs` (~70 lines) — `hybrid_sample`, `fisher_yates`.
- `shuffle/tests.rs` (~420 lines) — existing large test module; if still
  too big, split into `tests_normalize.rs`, `tests_dedup.rs`,
  `tests_sample.rs` mirroring the fn groups above.

## Tricky coupling

- `dedup.rs`'s `unique_track_count`/`dedup_by_similarity` call
  `similarity::build_similarity_groups` — needs
  `use super::similarity::build_similarity_groups;`.
- `similarity.rs`'s `build_similarity_groups` calls
  `super::normalize::{normalize_artist, normalize_title}`.
- All pure, no shared mutable state — low risk, straightforward function
  regrouping by concern (normalize / similarity-graph / dedup / sample).

## Verify after split

`cargo build -p qbz-mixtape`, `cargo test -p qbz-mixtape shuffle::` (this
file's ~35 tests, including the statistical seed-loop tests, must stay
green).
